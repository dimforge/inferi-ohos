import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(0x0000, 'inferi', 'Ability onCreate');
  }

  onDestroy(): void {
    hilog.info(0x0000, 'inferi', 'Ability onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    hilog.info(0x0000, 'inferi', 'Ability onWindowStageCreate');
    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(0x0000, 'inferi', 'Failed to load content: %{public}s', JSON.stringify(err));
        return;
      }
      hilog.info(0x0000, 'inferi', 'Content loaded');
    });
  }

  onWindowStageDestroy(): void {
    hilog.info(0x0000, 'inferi', 'Ability onWindowStageDestroy');
  }

  onForeground(): void {
    hilog.info(0x0000, 'inferi', 'Ability onForeground');
  }

  onBackground(): void {
    hilog.info(0x0000, 'inferi', 'Ability onBackground');
  }
}
