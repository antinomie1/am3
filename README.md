# am3

am3 是基于 [Aegle](../aegle) 的 Material 3 Expressive 控件库，是一个单独的 crate。它只负责控件本身：保留树、布局、输入、焦点、无障碍、主题切换、过渡与窗口都由 Aegle 提供。am3 也只通过 Aegle 的公开扩展接口实现，和第三方控件库走同一条路径：`Control` trait、`ControlKind` 皮肤、`handle!` 句柄与 `element!` 标记元素。

```rust
use am3::{Button, ButtonStyle, Mode, icons};

// 一个种子色生成浅色、深色与高对比主题，交给 Aegle 按系统偏好切换。
let seed = aegle::Color::rgb(0x67, 0x50, 0xA4);
let options = aegle::AppOptions {
    theme: am3::theme(seed, Mode::Light),
    dark_theme: Some(am3::theme(seed, Mode::Dark)),
    high_contrast_theme: Some(am3::theme(seed, Mode::DarkHighContrast)),
    ..Default::default()
};

// 控件挂到任意容器上。
let save = Button::new(&column, ButtonStyle::Filled, "保存")?;
save.set_icon(Some(icons::check()))?;
save.on_click(|_| Ok(()))?;
```

- [控件预览](docs/components.md)：每个控件在浅色与深色主题下的真实渲染截图（`cargo run --example gallery` 生成）。
- [API 设计](docs/api.md)：配色、令牌、控件句柄与扩展方式。
- [实现状态](docs/implementation.md)：已实现内容、验证方式与性能数据。

## 配色

配色只有一个输入：Aegle 的 `Theme`。`Scheme::of(&theme)` 由主题的 accent 求色相，按背景明度判断深浅、按前景对比判断高对比，生成 Material 的 47 个颜色角色（HCT 色彩空间、TonalSpot 方案，与 Material Color Utilities 的基准种子对齐到 ±2）。结果按线程缓存最近 6 个主题，皮肤每次绘制直接取用，所以：

- 应用切换浅色/深色/高对比主题、或给子树设置局部主题，am3 控件随之重新配色，不需要 am3 自己的状态；
- 任何 Aegle 主题（包括 `Theme::dark()`）都能得到一致的 Material 配色；
- `am3::theme(seed, mode)` 反过来从种子色生成 Aegle 主题，背景、表面、文字、强调色等字段取自同一方案。

应用内容可以用 `Role::Primary.token()` 等颜色令牌（`am3.color.primary`）绑定到当前方案。

## 许可

图标路径数据来自 Material Icons（Apache License 2.0）。
