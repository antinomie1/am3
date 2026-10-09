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

### 组合按钮

组合控件是普通的 Aegle 行容器加若干按钮类控件，句柄经 `Deref` 得到容器：

```rust
let group = ButtonGroup::new(&parent, GroupStyle::Connected, ButtonSize::Small)?;
let day = group.button(ButtonStyle::Tonal, "日")?;
group.button(ButtonStyle::Tonal, "周")?;
group.set_selection(Selection::Single)?;     // None | Single | Multiple

let split = SplitButton::new(&parent, ButtonStyle::Filled, "保存")?;
split.action().on_click(|_| Ok(()))?;
split.menu().on_click(|m| { /* m.selected()? == Some(true) 时显示菜单 */ Ok(()) })?;
split.set_menu_open(false)?;                  // 菜单关闭时复位

let seg = SegmentedButton::new(&parent, false)?; // true 为多选
seg.segment("日", None)?;
let chosen: Vec<usize> = seg.selected()?;

let menu = FabMenu::new(&parent, icons::add(), "新建", FabColor::PrimaryContainer)?;
menu.item(icons::edit(), "笔记")?.on_click(|_| Ok(()))?;
```

按钮之间的协作都在控件层完成，不需要 am3 的额外状态：单选组的互斥通过 Aegle 的 `Deferred` 在输入处理后取消兄弟按钮的选中；标准组的伸展由按下的按钮经 `Deferred` 设置自己与相邻按钮的伸展目标，各自在绘制时以弹簧采样，因此不触发布局。连接组、分割按钮与分段按钮的不对称圆角用路径绘制（按尺寸与半径缓存，仅在形变动画期间重建）；路径无法作为裁剪区域，因此这些形状上的按下以均匀状态层代替圆形涟漪。

## 选择控件与滑块

```rust
let all = Checkbox::new(&parent, "全选", false)?;
all.set_mixed(true)?;                      // 不确定状态，下一次改变时清除
all.set_error(true)?;
let small = Radio::new(&parent, "小", true)?;  // 同一父容器内互斥
let wifi = Switch::new(&parent, "Wi-Fi", true)?;
wifi.set_icons(Some(icons::check()), Some(icons::close()))?;
wifi.on_change(|s| { println!("{}", s.is_checked()?); Ok(()) })?;

let volume = Slider::new(&parent, 0.0, 100.0, 40.0)?;
volume.set_step(10.0, true)?;              // 步长与停止点
volume.set_size(SliderSize::Medium)?;
let price = Slider::new(&parent, 0.0, 1000.0, 800.0)?;
price.set_range(true)?;                    // 第二个手柄：start()/set_start()
```

复选框、单选按钮与开关共用一个控件 `SelectionControl`，行为复用 Aegle 的 `Toggle`；单选按钮的互斥同按钮组一样经 `Deferred` 完成，方向键由 am3 安装的 `am3::HOOKS` 处理（创建单选按钮时自动安装，与 Aegle 自己的钩子并存）。皮肤槽位：`foreground` 为标签，`indicator` 为选中容器（复选框方框、单选环与点、开关轨道），`border_color` 为未选中轮廓，`background` 为状态层，`caret` 为容器上的标记（对勾、开关手柄）。

滑块的每个手柄复用 Aegle 的 `Slider` 行为（指针捕获、方向键、Page、Home/End、语义增减与设值）；范围滑块按下时选较近的手柄，键盘移动最近操作的手柄，任何改变后保持起点不超过终点。程序设值时手柄以弹簧滑到新位置，拖动时直接跟手。

## 纸片与指示器

```rust
let vegan = Chip::new(&parent, ChipKind::Filter, "素食")?;   // 筛选纸片是切换按钮
let ada = Chip::new(&parent, ChipKind::Input, "Ada")?;
ada.set_leading(Some(icons::person()))?;
ada.on_remove(|chip| chip.remove())?;                     // 移除不算点击
Chip::new(&parent, ChipKind::Assist, "日程")?.set_elevated(true)?;

let download = Progress::linear(&parent, Some(0.4))?;
download.set_value(Some(0.6))?;                            // None 为不确定
download.set_wavy(true)?;
Progress::circular(&parent, None)?;
LoadingIndicator::new(&parent, true)?;                     // contained
```

纸片仍是 `PressableControl`：规格多了图标侧的内边距（有图标的一侧 8 dp，否则 16 dp）；筛选纸片的对勾出现或消失时，控件经 `Deferred` 让布局失效，因为宽度改变。指示器是只绘制的控件：线条与弧用圆头描边路径，波浪按 2 px 步长采样正弦；只有不确定状态、行进中的波浪与数值滑动期间请求帧。加载指示器的形状是在 48 个角度上采样的极坐标半径函数，变形即插值半径，轮廓是穿过采样点的闭合 Catmull-Rom 曲线——这是对 Material 圆角多边形变形的近似，不是逐顶点相同的实现。

## 容器与浮层

```rust
let card = Card::clickable(&parent, CardStyle::Outlined)?;     // Card 解引用为 Container
Text::new(&card, typescale::TITLE_MEDIUM, Role::on_surface, "标题")?;
card.on_click(|_| Ok(()))?;

let dialog = Dialog::new(&owner, Some(icons::delete()), "删除草稿？")?;
dialog.supporting("草稿将无法恢复。")?;
let delete = dialog.action("删除")?;
let closing = dialog.clone();
delete.on_click(move |_| closing.close())?;
dialog.show()?;                                                // Escape / scrim 也会关闭

let sheet = Sheet::bottom(&owner)?;                            // 或 Sheet::side、standard_*
let bar = Snackbar::new(&owner, "已归档")?;
bar.action("撤销")?;
bar.show(Some(am3::snackbar::SHORT))?;

let menu = Menu::new(&button)?;                                // 解引用为 aegle_widgets::Menu
menu.item("剪切")?.set_shortcut(Some("Ctrl+X"))?;
set_tooltip(&icon_button, Some("收藏"))?;
let list = List::new(&parent)?;
list.clickable_item("收件箱")?.leading_icon(icons::mail())?;
Badge::new(&icon_button)?.show_count(3)?;
```

`SurfaceControl` 是所有表面的控件：阴影、容器、描边、四个角各自的半径（不等时绘制缓存路径），可选按钮行为（状态层、涟漪、悬停升高）。卡片、对话框、表、提示条、富工具提示、列表与列表项都是它配不同的 `ControlKind`。`Text` 与 `IconView` 是带 Material 字体角色和颜色角色的文字与图标叶子控件。

模态层不是新的窗口机制，而是根节点下的一个全窗口宿主节点（scrim 控件），用 flex 放置其中的表面；显示时移为根的最后一个子节点，因此绘制在最上层。行为通过 Aegle 的 `Hooks` 加入，和 aegle-widgets 的弹出层同一方式：

- `overlay_at`：最上层的模态层覆盖整个窗口，命中只落在它的子树内；若其后还有显示中的根子节点（菜单、工具提示、提示条）且命中点在其中，则让给它们，所以对话框里的菜单照常可用；
- `key`：最上层模态层之后没有其他显示的根子节点时，Escape 请求关闭、Tab/Shift+Tab 在表面内循环；菜单打开时 Escape 先交给菜单；
- `wake`：提示条到时关闭，并与 Aegle 工具提示共享 `State::wake`；
- `removed`：忘记被移除的层。

所有关闭请求（Escape、scrim、超时）都变成宿主节点的动作，由拥有它的句柄在输入批次之后关闭，因此关闭动画与 `on_dismiss` 回调走同一条路径。进入与退出动画使用 `Node::animate`（透明度、缩放、位移）与 expressive 弹簧，退出动画结束时由 `on_transition_end` 隐藏宿主。

菜单与普通工具提示直接复用 aegle-widgets：`Menu` 只是在 Aegle 菜单弹出层上设置 `PANEL`、`MENU_ITEM` 的种类皮肤、主题覆盖（字号、内边距）与阴影，子菜单经 `Menu::submenu` 得到同样外观；`set_tooltip` 在根节点上设置 `TOOLTIP` 皮肤后调用 Aegle 的 `set_tooltip`。徽章是 Aegle 的 `Decorator`，不增加节点。

Aegle 在线性光空间混合颜色，而 Material 的状态层不透明度按 sRGB 混合定义。容器颜色已知的控件（实色按钮、菜单项）在 sRGB 中预先混合成不透明色；透明容器上的状态层（文字按钮、标准图标按钮、列表项）在深色背景上会比规范略明显。
