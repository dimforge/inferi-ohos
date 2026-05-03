import inferi from 'libinferi_ohos.so';
import fs from '@ohos.file.fs';
import { common } from '@kit.AbilityKit';
import resourceManager from '@ohos.resourceManager';

@Entry
@Component
struct Index {
  @State output: string = '';
  @State status: string = 'Ready';
  @State isRunning: boolean = false;
  @State modelLoaded: boolean = false;
  @State prompt: string = 'Hello';
  private pollTimer: number = -1;

  private getModelPath(): string {
    const context: common.UIAbilityContext = getContext(this) as common.UIAbilityContext;
    return `${context.filesDir}/model.gguf`;
  }

  private extractModelIfNeeded(): boolean {
    const modelPath: string = this.getModelPath();
    if (fs.accessSync(modelPath)) {
      return true;
    }
    this.status = 'Extracting from bundle...';
    const context: common.UIAbilityContext = getContext(this) as common.UIAbilityContext;
    const mgr: resourceManager.ResourceManager = context.resourceManager;
    const data: Uint8Array = mgr.getRawFileContentSync('model.gguf');
    const file: fs.File = fs.openSync(modelPath, fs.OpenMode.CREATE | fs.OpenMode.WRITE_ONLY);
    fs.writeSync(file.fd, data.buffer);
    fs.closeSync(file);
    return true;
  }

  private doLoadModel(): void {
    this.isRunning = true;
    try {
      if (!this.extractModelIfNeeded()) {
        this.isRunning = false;
        return;
      }
    } catch (e) {
      this.status = `Extract error: ${(e as Error).message}`;
      this.isRunning = false;
      return;
    }

    inferi.startLoadModel(this.getModelPath());

    this.pollTimer = setInterval(() => {
      this.status = inferi.getLoadStatus();
      if (inferi.isModelReady()) {
        clearInterval(this.pollTimer);
        this.modelLoaded = true;
        this.isRunning = false;
      } else if (this.status.startsWith('ERROR')) {
        clearInterval(this.pollTimer);
        this.isRunning = false;
      }
    }, 500);
  }

  private doGenerate(): void {
    this.isRunning = true;
    this.output = '';

    inferi.startGeneration(this.prompt);

    this.pollTimer = setInterval(() => {
      this.status = inferi.getGenStatus();
      this.output = inferi.getGenOutput();
      if (inferi.isGenDone()) {
        clearInterval(this.pollTimer);
        this.isRunning = false;
      }
    }, 500);
  }

  build() {
    Scroll() {
      Column() {
        Image($rawfile('inferi-logo.png'))
          .width('60%')
          .objectFit(ImageFit.Contain)
          .margin({ top: 12, bottom: 2 })

        Text(this.status)
          .fontSize(9)
          .fontColor(this.isRunning ? '#FF8800' : '#007DFF')
          .margin({ bottom: 6 })
          .textAlign(TextAlign.Center)
          .width('90%')

        if (!this.modelLoaded) {
          Button('Load Model')
            .width('70%')
            .height(28)
            .fontSize(12)
            .margin({ bottom: 4 })
            .enabled(!this.isRunning)
            .onClick(() => {
              this.doLoadModel();
            })
        }

        TextInput({ text: this.prompt, placeholder: 'Enter prompt' })
          .width('90%')
          .height(28)
          .fontSize(10)
          .margin({ bottom: 4 })
          .enabled(!this.isRunning)
          .onChange((value: string) => {
            this.prompt = value;
          })

        Button('Generate')
          .width('70%')
          .height(28)
          .fontSize(12)
          .margin({ bottom: 4 })
          .enabled(!this.isRunning && this.modelLoaded)
          .onClick(() => {
            this.doGenerate();
          })

        Text(this.output)
          .fontSize(10)
          .fontColor('#333')
          .textAlign(TextAlign.Start)
          .width('90%')
          .padding(4)
          .backgroundColor('#F5F5F5')
          .borderRadius(4)
          .margin({ bottom: 60 })
      }
      .width('100%')
      .alignItems(HorizontalAlign.Center)
    }
    .width('100%')
    .height('100%')
    .backgroundColor('#FFFFFF')
  }
}
