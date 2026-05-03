use napi_derive_ohos::napi;
use std::sync::{Arc, Mutex};

use inferi::context::{LlmContext, LlmOps};
use inferi::gguf::{Gguf, GgufMetadataValue};
use inferi::models::llama2::cpu::Llama2Config;
use inferi::models::llama2::{Llama2, Llama2State, Llama2Weights, LlamaModelType};
use inferi::models::sampler::Sampler;
use inferi::models::tokenizers::{Gpt2Tokenizer, LlamaTokenizer};
use inferi::re_exports::vortx::shapes::TensorLayoutBuffers;
use inferi::tensor_cache::TensorCache;
use khal::backend::{Backend, GpuBackend};
use nalgebra::DVector;

// ---------------------------------------------------------------------------
// Tokenizer wrapper
// ---------------------------------------------------------------------------
enum AnyTokenizer {
    Llama(LlamaTokenizer),
    Gpt2(Gpt2Tokenizer),
}

impl AnyTokenizer {
    fn from_gguf(gguf: &Gguf) -> Result<Self, String> {
        let tok_type = gguf.metadata.get("tokenizer.ggml.model")
            .ok_or_else(|| "Missing tokenizer.ggml.model".to_string())?;
        let tok_type = if let GgufMetadataValue::String(s) = tok_type { s.as_str() } else { "" };
        match tok_type {
            "gpt2" => Ok(Self::Gpt2(Gpt2Tokenizer::from_gguf(gguf))),
            "llama" => Ok(Self::Llama(LlamaTokenizer::from_gguf(gguf))),
            other => Err(format!("Unknown tokenizer type: {}", other)),
        }
    }
    fn eos(&self) -> usize {
        match self { Self::Llama(t) => t.eos(), Self::Gpt2(t) => t.eos() }
    }
    fn bos_str(&self) -> String {
        match self { Self::Llama(t) => t.bos_str().to_string(), Self::Gpt2(t) => t.bos_str().to_string() }
    }
    fn eos_str(&self) -> String {
        match self { Self::Llama(t) => t.eos_str().to_string(), Self::Gpt2(t) => t.eos_str().to_string() }
    }
    fn encode(&self, text: &str, bos: bool, eos: bool) -> Vec<usize> {
        match self {
            Self::Llama(t) => t.encode(text, bos, eos),
            Self::Gpt2(t) => t.encode(text),
        }
    }
    fn decode(&self, prev: usize, tok: usize) -> String {
        match self {
            Self::Llama(t) => t.decode(prev, tok),
            Self::Gpt2(t) => t.decode(&[tok as u32]),
        }
    }
}

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------
struct InferenceState {
    backend: Arc<GpuBackend>,
    ops: LlmOps,
    transformer: Llama2,
    weights: Llama2Weights,
    tokenizer: AnyTokenizer,
    sampler: Sampler,
    config: Llama2Config,
    state: Llama2State,
    shapes: TensorLayoutBuffers,
    tensor_cache: TensorCache,
}

static STATE: Mutex<Option<InferenceState>> = Mutex::new(None);
static CHAT_TEMPLATE: Mutex<Option<String>> = Mutex::new(None);
static LOAD_STATUS: Mutex<String> = Mutex::new(String::new());
static MODEL_READY: Mutex<bool> = Mutex::new(false);

fn set_status(s: &str) {
    *LOAD_STATUS.lock().unwrap() = s.to_string();
}

// ---------------------------------------------------------------------------
// Model loading (background thread with async runtime)
// ---------------------------------------------------------------------------
#[napi]
pub fn start_load_model(path: String) {
    *MODEL_READY.lock().unwrap() = false;
    set_status("Reading model file...");

    std::thread::spawn(move || {
        async_std::task::block_on(async move {
            // Init CPU backend.
            let backend = Arc::new(GpuBackend::Cpu);

            // Read file.
            set_status("Reading model file...");
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) => { set_status(&format!("ERROR: {}", e)); return; }
            };
            let mb = bytes.len() / (1024 * 1024);
            set_status(&format!("Read {} MB. Parsing GGUF...", mb));

            let gguf = match Gguf::from_bytes(&bytes) {
                Ok(g) => g,
                Err(e) => { set_status(&format!("ERROR: {:?}", e)); return; }
            };
            drop(bytes);

            let model_type = match gguf.metadata.get("general.architecture") {
                Some(GgufMetadataValue::String(name)) if name.to_lowercase().contains("qwen2") => LlamaModelType::Qwen2,
                _ => LlamaModelType::Llama,
            };

            let mut config = Llama2Config::from_gguf_with_model_type(&gguf, model_type);
            config.max_position_embeddings = config.max_position_embeddings.min(512);

            set_status("Building tokenizer...");
            let tokenizer = match AnyTokenizer::from_gguf(&gguf) {
                Ok(t) => t,
                Err(e) => { set_status(&format!("ERROR: {}", e)); return; }
            };

            let chat_template_str = gguf.metadata.get("tokenizer.chat_template")
                .map(|val| {
                    if let GgufMetadataValue::String(s) = val { s.clone() }
                    else { LlamaTokenizer::CHAT_TEMPLATE.to_string() }
                })
                .unwrap_or_else(|| LlamaTokenizer::CHAT_TEMPLATE.to_string())
                .replace(".split(", "|split(")
                .replace("[-1]", "|last");

            set_status("Creating pipeline...");
            let ops = match LlmOps::new(&backend) {
                Ok(o) => o,
                Err(e) => { set_status(&format!("ERROR LlmOps: {:?}", e)); return; }
            };
            let transformer = match Llama2::new(&backend, model_type) {
                Ok(t) => t,
                Err(e) => { set_status(&format!("ERROR Llama2: {:?}", e)); return; }
            };

            set_status(&format!("Loading {} layers...", config.num_hidden_layers));
            let weights = match Llama2Weights::from_gguf(&backend, &config, &gguf) {
                Ok(w) => w,
                Err(e) => { set_status(&format!("ERROR weights: {:?}", e)); return; }
            };
            drop(gguf);

            let gpu_state = match Llama2State::new(&backend, &config) {
                Ok(s) => s,
                Err(e) => { set_status(&format!("ERROR state: {:?}", e)); return; }
            };

            let sampler = Sampler::new(config.vocab_size, 0.7, 0.95);
            let shapes = TensorLayoutBuffers::new(&backend);
            let tensor_cache = TensorCache::default();

            *CHAT_TEMPLATE.lock().unwrap() = Some(chat_template_str);
            *STATE.lock().unwrap() = Some(InferenceState {
                backend,
                ops,
                transformer,
                weights,
                tokenizer,
                sampler,
                config,
                state: gpu_state,
                shapes,
                tensor_cache,
            });
            *MODEL_READY.lock().unwrap() = true;
            set_status(&format!("Ready — {}, {} layers, vocab {}",
                model_type.gguf_model_name(), config.num_hidden_layers, config.vocab_size));
        });
    });
}

#[napi]
pub fn get_load_status() -> String { LOAD_STATUS.lock().unwrap().clone() }

#[napi]
pub fn is_model_ready() -> bool { *MODEL_READY.lock().unwrap() }

// ---------------------------------------------------------------------------
// Generation (background thread)
// ---------------------------------------------------------------------------
static GEN_OUTPUT: Mutex<String> = Mutex::new(String::new());
static GEN_STATUS: Mutex<String> = Mutex::new(String::new());
static GEN_DONE: Mutex<bool> = Mutex::new(true);

#[napi]
pub fn start_generation(prompt: String) {
    *GEN_OUTPUT.lock().unwrap() = String::new();
    *GEN_STATUS.lock().unwrap() = "Starting...".to_string();
    *GEN_DONE.lock().unwrap() = false;

    std::thread::spawn(move || {
        async_std::task::block_on(async move {
            let chat_template_str = match CHAT_TEMPLATE.lock().unwrap().clone() {
                Some(t) => t,
                None => { *GEN_STATUS.lock().unwrap() = "ERROR: model not loaded".into(); *GEN_DONE.lock().unwrap() = true; return; }
            };

            let mut state_guard = STATE.lock().unwrap();
            let s = match state_guard.as_mut() {
                Some(s) => s,
                None => { *GEN_STATUS.lock().unwrap() = "ERROR: model not loaded".into(); *GEN_DONE.lock().unwrap() = true; return; }
            };

            // Render prompt.
            let bos_str = s.tokenizer.bos_str();
            let eos_str = s.tokenizer.eos_str();
            let rendered = {
                use minijinja::{context, Environment};
                let mut env = Environment::new();
                env.set_trim_blocks(true);
                env.add_global("bos_token", bos_str);
                env.add_global("eos_token", eos_str);
                env.add_global("add_generation_prompt", true);
                env.add_template("main", &chat_template_str).unwrap();
                let tmpl = env.get_template("main").unwrap();
                let messages = vec![context!(role => "user", content => &prompt)];
                tmpl.render(context!(messages => messages)).unwrap()
            };

            let prompt_toks = s.tokenizer.encode(&rendered, false, false);
            *GEN_STATUS.lock().unwrap() = format!("Prompt: {} tokens", prompt_toks.len());

            let mut token = prompt_toks[0];
            let start = std::time::Instant::now();
            let mut gen_count: usize = 0;
            let mut logits = DVector::zeros(s.config.vocab_size);

            for pos in 0..512usize {
                // Forward pass through the abstracted GPU/CPU backend.
                let forward_result = forward_logits(s, pos as u32, token as u32, &mut logits).await;
                if let Err(e) = forward_result {
                    *GEN_STATUS.lock().unwrap() = format!("ERROR forward: {:?}", e);
                    break;
                }

                let is_prefill = pos < prompt_toks.len() - 1;
                let next = if is_prefill {
                    prompt_toks[pos + 1]
                } else {
                    s.sampler.sample(&mut logits)
                };

                if is_prefill {
                    *GEN_STATUS.lock().unwrap() = format!("prefill {}/{}", pos + 1, prompt_toks.len());
                } else {
                    if next == s.tokenizer.eos() { break; }
                    gen_count += 1;
                    let tok_str = s.tokenizer.decode(token, next);
                    GEN_OUTPUT.lock().unwrap().push_str(&tok_str);
                    let elapsed = start.elapsed().as_secs_f64();
                    let tps = if elapsed > 0.0 { gen_count as f64 / elapsed } else { 0.0 };
                    *GEN_STATUS.lock().unwrap() = format!("gen {} ({:.2} tok/s)", gen_count, tps);
                }

                token = next;
            }

            let elapsed = start.elapsed().as_secs_f64();
            let tps = if elapsed > 0.0 { gen_count as f64 / elapsed } else { 0.0 };
            *GEN_STATUS.lock().unwrap() = format!("done — {} tokens, {:.2} tok/s", gen_count, tps);
            *GEN_DONE.lock().unwrap() = true;
        });
    });
}

/// Forward pass — mirrors ChatLlama2::forward_logits using the backend abstraction.
async fn forward_logits(
    s: &mut InferenceState,
    pos: u32,
    token: u32,
    out: &mut DVector<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    s.shapes.clear_tmp();

    let (rope_config, rms_norm_config, attn_params) = s.config.derived_configs(pos);

    let mut encoder = s.backend.begin_encoding();
    s.backend.write_buffer(s.state.rope_config_mut().buffer_mut(), 0, &[rope_config])?;
    s.backend.write_buffer(s.state.rms_norm_config_mut().buffer_mut(), 0, &[rms_norm_config])?;
    s.backend.write_buffer(s.state.attn_params_mut().buffer_mut(), 0, &[attn_params])?;
    s.state.x.copy_from_view(&mut encoder, s.weights.token_embd.row(token))?;
    s.backend.submit(encoder)?;

    let mut ctxt = LlmContext {
        backend: &s.backend,
        shapes: &mut s.shapes,
        cache: &mut s.tensor_cache,
        pass: None,
        encoder: None,
        ops: &s.ops,
    };
    ctxt.begin_submission();
    s.transformer.launch(&mut ctxt, &mut s.state, &s.weights, &s.config, &attn_params, pos)?;
    drop(ctxt.pass.take());

    let (logits, readback) = s.state.logits_and_readback_mut();
    readback.copy_from_view(ctxt.encoder.as_mut().unwrap(), logits)?;
    ctxt.submit();

    s.backend.read_buffer(s.state.logits_readback().buffer(), out.as_mut_slice()).await?;

    Ok(())
}

#[napi]
pub fn get_gen_status() -> String { GEN_STATUS.lock().unwrap().clone() }

#[napi]
pub fn get_gen_output() -> String { GEN_OUTPUT.lock().unwrap().clone() }

#[napi]
pub fn is_gen_done() -> bool { *GEN_DONE.lock().unwrap() }
