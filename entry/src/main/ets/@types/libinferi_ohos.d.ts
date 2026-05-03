declare namespace inferi {
  function startLoadModel(path: string): void;
  function getLoadStatus(): string;
  function isModelReady(): boolean;
  function startGeneration(prompt: string): void;
  function getGenStatus(): string;
  function getGenOutput(): string;
  function isGenDone(): boolean;
}

export default inferi;
