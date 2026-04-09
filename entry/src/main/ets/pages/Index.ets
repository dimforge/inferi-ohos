import inferi from 'libinferi_ohos.so';

@Entry
@Component
struct Index {
  @State greeting: string = '';
  @State fibResult: string = '';
  @State addResult: string = '';
  @State reverseResult: string = '';
  @State fibInput: number = 10;

  build() {
    Scroll() {
      Column() {
        Text('Inferi')
          .fontSize(18)
          .fontWeight(FontWeight.Bold)
          .margin({ top: 16, bottom: 4 })

        Text('Rust on HarmonyOS')
          .fontSize(11)
          .fontColor('#666')
          .margin({ bottom: 12 })

        Button('Greet')
          .width('70%')
          .height(32)
          .fontSize(13)
          .margin({ bottom: 4 })
          .onClick(() => {
            this.greeting = inferi.greet('World');
          })

        Text(this.greeting)
          .fontSize(11)
          .fontColor('#007DFF')
          .textAlign(TextAlign.Center)
          .margin({ bottom: 10 })

        Button('17 + 25')
          .width('70%')
          .height(32)
          .fontSize(13)
          .margin({ bottom: 4 })
          .onClick(() => {
            const result: number = inferi.add(17, 25);
            this.addResult = `= ${result}`;
          })

        Text(this.addResult)
          .fontSize(11)
          .margin({ bottom: 10 })

        Button(`fib(${this.fibInput})`)
          .width('70%')
          .height(32)
          .fontSize(13)
          .margin({ bottom: 4 })
          .onClick(() => {
            const result: number = inferi.fibonacci(this.fibInput);
            this.fibResult = `= ${result}`;
            this.fibInput += 5;
          })

        Text(this.fibResult)
          .fontSize(11)
          .margin({ bottom: 10 })

        Button('Reverse')
          .width('70%')
          .height(32)
          .fontSize(13)
          .margin({ bottom: 4 })
          .onClick(() => {
            this.reverseResult = inferi.reverseString('HarmonyOS');
          })

        Text(this.reverseResult)
          .fontSize(11)
          .margin({ bottom: 10 })
      }
      .width('100%')
      .alignItems(HorizontalAlign.Center)
    }
    .width('100%')
    .height('100%')
  }
}
