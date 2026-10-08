# API 设计

## 边界

am3 只提供控件、配色方案与设计令牌。树、布局、输入路由、焦点、无障碍导出、主题继承、过渡、弹出层与窗口都属于 Aegle；am3 不复制这些机制，也不包一层自己的应用框架。am3 的控件通过 Aegle 的公开扩展接口实现，和第三方控件库完全相同：

| Aegle 接口 | am3 的用法 |
|---|---|
| `Control` trait | 每类控件一个结构体（如 `PressableControl`），负责行为、测量、绘制与语义 |
| `ControlKind` + 皮肤函数 | 每个变体一个 `static` 种类，皮肤是 `fn(&Theme, VisualState) -> Appearance`，从 `Scheme::of(theme)` 取色 |
| `handle!` | 类型化句柄（`Button`、`IconButton`、`Fab` …）及其样式组 setter |
| `Container::add` | 句柄构造函数把控件挂到任意容器 |
| `Control::text_role` | 控件把节点文字样式换算为自己的 Material 字体角色 |
| `TransitionProperty::Paint` | 新控件的颜色变化使用 expressive 默认效果弹簧 |

因此应用可以用 Aegle 的 `set_kind_skin`、`set_skin`、局部样式（`set_background` 等）、局部主题和颜色令牌修改 am3 控件，与修改内置控件的方式一样。

## 配色

```rust
pub enum Mode { Light, Dark, LightHighContrast, DarkHighContrast }
pub struct Scheme { pub primary: Color, pub on_primary: Color, /* … 共 47 个角色 */ }

impl Scheme {
    pub fn from_seed(seed: Color, mode: Mode) -> Scheme;
    pub fn from_hue(hue: f64, mode: Mode) -> Scheme;
    pub fn of(theme: &Theme) -> Scheme;        // 皮肤用：从任意 Aegle 主题推导，按线程缓存
    pub fn role(&self, role: Role) -> Color;
}
pub fn theme(seed: Color, mode: Mode) -> Theme; // 生成 Aegle 主题
impl Role { pub fn token(self) -> Token<Color>; } // am3.color.<角色> 颜色令牌
```

设计取舍：方案不作为 am3 的全局状态存在，而是 Aegle 主题的纯函数。主题切换、子树主题、高对比都已经由 Aegle 处理，am3 只需在绘制时由主题求方案；缓存键是主题的 accent、背景与前景三个颜色，6 项循环替换，命中时只比较 3 个 u32。

色彩计算使用 HCT（CAM16 色相与彩度、L* 明度）：主色调色板色相取自 accent，彩度 36；secondary 16；tertiary 色相 +60°、彩度 24；中性 6、中性变体 8；error 固定色相 25、彩度 84。各角色的色调表按模式给出（浅色 primary 40、深色 80、高对比浅色 25、高对比深色 92 等），见 `src/color/scheme.rs`。

## 设计令牌

`am3::tokens` 提供 Material 的非颜色令牌，均为常量：

- `typescale::*`：15 个基准字体角色（`TypeStyle { size, line_height, weight, tracking }`），以 14 px 正文为基准按主题字号缩放，`emphasized()` 给出 expressive 的强调字重；
- `shape::*`：圆角刻度 0/4/8/12/16/20/28/32/48 与 `Corner::Full`；
- `elevation(level, shadow)` 与 `level_dp`：0–5 级高度对应 0/1/3/6/8/12 dp 的阴影；
- `motion::*`：expressive 弹簧（空间 0.6/800、0.8/380、0.8/200，效果 1/3800、1/1600、1/800，阻尼比/刚度），`motion::standard::*` 为标准方案；`Spring::aegle()` 转为 Aegle 的阻尼系数弹簧，`transition()` 转为过渡；
- `state::*`：状态层不透明度（悬停 0.08、焦点 0.10、按下 0.10、拖动 0.16）与禁用不透明度（内容 0.38、容器 0.12）。

## 按钮类控件

按钮、图标按钮、FAB 共用一个控件 `PressableControl`：容器、阴影、描边、涟漪与居中的图标/标签/尾随图标。它持有组件与变体（内部 `Look`），据此给出 `ControlKind`（决定颜色）与尺寸规格（高度、宽度、内边距、图标、字体角色、圆角、按下与选中圆角、描边宽度、高度等级）。圆角形变与高度变化由控件自己以 expressive 弹簧采样帧时间，只在运动期间请求帧；减少动态效果时直接到位。

```rust
let b = Button::new(&parent, ButtonStyle::Tonal, "标签")?;
b.set_size(ButtonSize::Medium)?;          // ExtraSmall | Small | Medium | Large | ExtraLarge
b.set_shape(ButtonShape::Square)?;        // Round | Square
b.set_icon(Some(icons::add()))?;
b.set_selected(Some(false))?;             // 变为切换按钮；None 恢复普通按钮
b.on_click(|b| { println!("{:?}", b.selected()?); Ok(()) })?;

let i = IconButton::new(&parent, IconStyle::Filled, icons::favorite(), "收藏")?;
i.set_width(IconWidth::Wide)?;            // 标签不显示，作为无障碍名称

let f = Fab::extended(&parent, icons::edit(), "撰写")?;
f.set_color(FabColor::Tertiary)?;
f.set_extended(false)?;                   // 收起为图标
```

所有按钮类句柄共享 `set_text`、`text`、`set_icon`、`set_trailing_icon`、`set_selected`、`selected`、`activate`、`on_click`、`clear_on_click`，以及 Aegle 的 `Node` 方法（`set_enabled`、`focus`、`set_background` 等，经 `Deref`）。切换按钮激活时翻转选中并报告 `Action::Change`，`on_click` 回调同样收到。无障碍角色为 Button，切换按钮带 toggled 状态。

颜色来自皮肤，几何来自规格：应用改颜色用 Aegle 样式或皮肤，改尺寸用 am3 的尺寸枚举。固定宽度的控件（图标按钮、FAB）设置明确的布局宽度，不会在列中被拉伸；内容宽度的按钮在列中按 Aegle 的 flex 规则拉伸，需要时设置 `align_items`。
