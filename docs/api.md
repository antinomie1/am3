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

## 导航与应用结构

```rust
let bar = NavigationBar::new(&root)?;
let mail = bar.item(icons::mail(), "邮件")?;       // 第一个目的地自动选中
mail.badge()?.show_count(3)?;
mail.on_click(|item| { /* 切换页面 */ Ok(()) })?;

let rail = NavigationRail::new(&row)?;
Fab::new(rail.header()?, icons::edit(), "写邮件")?;
rail.item(icons::mail(), "收件箱")?;
rail.set_expanded(true)?;

let drawer = NavigationDrawer::modal(&root)?;      // 或 standard(&parent)
drawer.headline("邮件")?;
drawer.item(icons::mail(), "收件箱")?;
drawer.show()?;

let tabs = Tabs::new(&column, TabStyle::Primary)?;
tabs.tab(Some(icons::send()), "航班")?;

let top = TopAppBar::new(&column, AppBarStyle::Medium, "标题")?;
top.navigation(icons::arrow_back(), "返回")?;
top.action(icons::more_vert(), "更多")?;
top.set_scrolled(true)?;

let toolbar = Toolbar::floating(&parent, ToolbarColor::Vibrant, false)?;
toolbar.icon_button(icons::edit(), "编辑")?;
```

导航栏、栏杆与抽屉共用 `NavItemControl`：堆叠（指示器下方标签）或行内（胶囊）两种排布，选中时在 `Deferred` 中取消同一导航组件内其他目的地（组件根节点下的全部 `NavItemControl`，可跨抽屉的分节）。栏杆展开/收起只是切换排布并按新的字体角色重排标签。目的地的徽章共享控件上次绘制的图标位置，所以无论排布如何都贴在图标右上角。

标签页的指示器由 `TabRowControl` 绘制：它需要选中标签的布局位置，因此通过 Aegle 的 `place` 钩子在每次几何更新后读取选中标签的边界，写入行控件的目标，绘制时以弹簧滑过去；选择改变只标记几何脏，不额外遍历。

应用栏与工具栏是 `SurfaceControl` 配不同种类；鲜明工具栏在自身子树上为标准图标按钮与文字按钮设置种类皮肤（on primary container）。为此 Aegle 修正了一个问题：控件类型随状态改变（如图标按钮换样式、切换按钮选中）时，子树的种类皮肤会按新类型重新解析（Aegle 提交 b201d1d）。

## 输入

```rust
let name = TextField::new(&column, FieldStyle::Outlined, "姓名")?;
name.set_supporting(Some("与证件一致"))?;
name.set_leading_icon(Some(icons::person()))?;
name.set_clearable(true)?;
name.on_change(|field| { println!("{}", field.text()?); Ok(()) })?;
let bio = TextField::multiline(&column, FieldStyle::Filled, "简介")?;
bio.set_counter(Some(120))?;

let search = Search::new(&column, "搜索邮件")?;    // 输入时打开结果视图
List::new(search.results())?.clickable_item("Lisbon")?;

let date = DatePicker::modal(&root)?;              // 或 docked(&parent)
date.set_selected(Date::new(2026, 8, 17))?;
date.on_confirm(|picker| { let _ = picker.selected()?; Ok(()) })?;
date.show()?;

let time = TimePicker::inline(&column)?;           // 或 modal(&root)
time.set_24_hour(true)?;
time.set_time(19, 30)?;

let carousel = Carousel::new(&column, 180.0, 200.0)?;
carousel.item(true)?.on_click(|_| Ok(()))?;
```

`TextField` 不是新的文本控件：`FieldControl` 包着 `aegle_controls::TextField`（编辑、输入法、选择、无障碍），只负责外框、标签、图标、辅助文字与计数的绘制和尺寸。为此 Aegle 增加了两个通用接口（Aegle 提交 14d23fe）：`Control::text_viewport`，让外框控件声明编辑器实际可用的文字区域（光标滚动据此保持在图标之间）；`Editor::changes()`，不清空地读取编辑器的变更，外框据此报告改变并更新计数。

日期与时间选择器各是一个绘制整块网格或表盘的控件（`CalendarControl`、`DialControl`），而不是几十个按钮节点：命中、光标与范围带都在控件内计算，节点数与日期多少无关。`Date` 是不依赖外部库的公历日期（`days`/`from_days` 为距 1970-01-01 的天数），`Date::new` 是 `const fn`，可写常量。搜索视图是 Aegle 的 `Popup`，轮播是一个视口控件，把滚轮的竖向分量转为横向滚动。

## 标记元素

全部控件都有标记元素，名字为 `Md` 加控件名。它们就是普通的 `element!` 定义，和任何第三方库同一条路径：编译期 `ui!` 经 `use am3::MdButton;` 找到规格（该导入同时带来标记类型与隐藏的规格宏），运行时加载用 `am3::elements()` 注册全部元素，可与其他库的 `Elements` 合并。`id` 字段的类型是 am3 的句柄（`MdTextField` 的 `id` 是 `am3::TextField`），处理块里的 `self.checked` 等字段来自 `get` 声明。

| 元素 | 构造属性 | 可绑定属性 | 事件 / `self` 字段 |
| --- | --- | --- | --- |
| `MdButton` | `text`、`style`（filled/tonal/outlined/elevated/text） | `text`、`style`、`size`、`square`、`icon`、`selected` | `clicked` / `selected` |
| `MdIconButton` | `icon`、`label`、`style`（standard/filled/tonal/outlined） | `icon`、`style`、`size`、`width`、`selected`、`badge` | `clicked` / `selected` |
| `MdFab` | `icon`、`label`、`extended` | `extended`、`size`、`color` | `clicked` |
| `MdFabMenu` › `MdFabMenuItem` | `icon`、`label`、`color` › `icon`、`text` | `open` | 菜单项 `clicked` / `open` |
| `MdButtonGroup` › `MdGroupButton` | `style`、`size`、`selection` › `text`、`style` | 按钮 `selected` | 按钮 `clicked` / `selected` |
| `MdSplitButton` | `text`、`style` | `size`、`menu_open` | `clicked`、`menu` |
| `MdSegmentedButton` › `MdSegment` | `multiple` › `text`、`icon` | 分段 `selected` | 分段 `clicked` / `selected`（按钮的 `selected` 为首个选中序号） |
| `MdCheckbox`、`MdRadio`、`MdSwitch` | `text`、`checked` | `text`、`checked`；复选框 `mixed`、`error`；开关 `icons` | `changed` / `checked` |
| `MdSlider` | `min`、`max`、`value`、`step`、`ticks` | `value`、`size`、`centered` | `changed` / `value` |
| `MdChip` | `text`、`kind`（assist/filter/input/suggestion） | `text`、`icon`、`elevated`、`selected` | `clicked`、`removed` / `selected` |
| `MdProgress`、`MdLoadingIndicator` | `circular`、`indeterminate`；`contained` | `value`、`wavy` | |
| `MdText`、`MdIcon`、`MdDivider` | `text`、`typescale`、`color`；`icon`、`size`、`color`；`vertical` | `text`、`color`；`icon`、`color` | `text` |
| `MdCard` | `style`、`clickable` | | `clicked` |
| `MdList` › `MdListItem` | › `headline`、`clickable`、`supporting`、`leading`、`trailing`、`trailing_text` | | 列表项 `clicked` |
| `MdDialog` › `MdDialogAction` | `headline`、`icon` › `text` | `headline`、`open` | `dismissed`；操作 `clicked` |
| `MdSheet` | `kind`（bottom/side/standard_bottom/standard_side）、`headline` | `open` | `dismissed` |
| `MdSnackbar` › `MdSnackbarAction` | `text`、`closable`、`duration`（毫秒，0 为不自动关闭） › `text` | `text`、`open` | `timeout`；操作 `clicked` |
| `MdNavigationBar`、`MdNavigationRail`、`MdNavigationDrawer` › `MdNavItem` | 栏杆 `expanded`；抽屉 `modal`、`headline` › `icon`、`text` | 栏杆 `expanded`；抽屉 `open`；目的地 `selected`、`selected_icon`、`badge` | 目的地 `clicked` / `selected`（容器为选中序号） |
| `MdTabs` › `MdTab` | `style`（primary/secondary） › `text`、`icon` | 标签 `selected` | 标签 `clicked` / `selected` |
| `MdTopAppBar`、`MdBottomAppBar`、`MdToolbar` › `MdAction` | `title`、`style`、`subtitle`；工具栏 `floating`、`vertical`、`vibrant` › `icon`、`label`、`navigation` | `title`、`scrolled`；操作 `icon` | 操作 `clicked` |
| `MdTextField` | `label`、`style`（filled/outlined）、`multiline` | `text`、`placeholder`、`supporting`、`error`、`leading_icon`、`trailing_icon`、`clearable`、`counter`、`password`、`read_only` | `changed` / `text` |
| `MdSearch` | `placeholder` | `trailing_icon` | `changed` / `text`；子元素放入搜索视图 |
| `MdDatePicker` | `modal`、`range` | `selected`（ISO 8601）、`open` | `changed`、`confirmed` / `selected`、`end` |
| `MdTimePicker` | `modal`、`h24` | `hour`、`minute`、`h24`、`open` | `changed`、`confirmed` / `hour`、`minute` |
| `MdCarousel` › `MdCarouselItem` | `item_width`、`height` › `clickable` | | 项目 `clicked` |

`›` 表示父子元素：子元素只能写在父元素内（`parent` 约束在挂载前检查），`MdNavItem` 与 `MdAction` 可用于三种容器，放错位置时构建失败。选项属性写蛇形命名的标识符；图标按 `icons` 中的名称写字符串（`icons::named`），颜色按角色名写（`"on_surface_variant"`），这两者在构建时检查。`badge` 为 -1 显示小圆点、0 隐藏、正数显示计数。

子元素需要父组件的句柄（标签页要加到 `Tabs`，导航目的地要加到栏、栏杆或抽屉）。标记引擎只把父元素的容器节点交给子元素，所以 am3 在 `State::ext` 里按节点记录这些组合句柄（弱引用节点，不形成环），由 `removed` 钩子在节点删除时清除；标记设置的徽章同样按节点复用，绑定反复更新时不叠加装饰器。

菜单与富工具提示锚定在另一个控件上，暂无标记元素，用 Rust 句柄创建；导航栏杆顶部的 FAB 也只能在 Rust 中放入 `header()`。
