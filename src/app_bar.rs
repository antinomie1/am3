//! App bars. A top app bar carries the screen's title with a navigation
//! button before it and actions after it: small (64 dp), center-aligned,
//! and the flexible medium (112 dp, headline medium) and large (120 dp,
//! display small) bars whose title sits on its own row. Scrolled content
//! under it turns it to the surface container. The bottom app bar holds
//! actions and a FAB along the bottom edge.

use aegle_ui::{Align, Container, Insets, Length, Result};

use crate::{
    Fab, Icon, IconButton, IconStyle, Role, Text,
    skin::{Paint, kinds},
    surface::{self, Part, SurfaceControl},
    tokens::{TypeStyle, typescale},
};

kinds! { container
    /// A top app bar over unscrolled content.
    TOP_APP_BAR = "TopAppBar", top => |s, _on| Paint::new(s.surface, s.on_surface);
    /// A top app bar over scrolled content.
    TOP_APP_BAR_SCROLLED = "ScrolledTopAppBar", scrolled => |s, _on| {
        Paint::new(s.surface_container, s.on_surface)
    };
    /// A bottom app bar.
    BOTTOM_APP_BAR = "BottomAppBar", bottom => |s, _on| {
        Paint::new(s.surface_container, s.on_surface)
    };
}

/// The top app bar variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarStyle {
    /// Title after the navigation button.
    #[default]
    Small,
    /// Title centered in the bar.
    CenterAligned,
    /// Title on its own row in headline medium.
    Medium,
    /// Title on its own row in display small.
    Large,
}

/// A top app bar.
#[derive(Clone)]
pub struct TopAppBar {
    bar: Container,
    navigation: Container,
    titles: Container,
    actions: Container,
    title: Text,
    style: AppBarStyle,
}

impl std::ops::Deref for TopAppBar {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.bar
    }
}

impl TopAppBar {
    /// Appends a bar showing `title`, as wide as its parent allows.
    pub fn new(parent: &Container, style: AppBarStyle, title: &str) -> Result<Self> {
        let control = SurfaceControl::new(&TOP_APP_BAR, Part::Pane, 0.0, 0.0);
        let bar = surface::add(parent, control, 0.0)?;
        bar.set_align_self(Some(Align::Stretch))?;
        let top = bar.row()?;
        top.set_height(64.0)?;
        top.set_align_items(Some(Align::Center))?;
        top.set_gap(0.0)?;
        top.set_padding(crate::edges(4.0, 0.0, 4.0, 0.0))?;
        let navigation = top.row()?;
        navigation.set_visible(false)?;
        let (role, titles) = match style {
            AppBarStyle::Small | AppBarStyle::CenterAligned => {
                let titles = top.column()?;
                titles.set_grow(1.0)?;
                titles.set_gap(0.0)?;
                titles.set_padding(crate::edges(12.0, 0.0, 12.0, 0.0))?;
                if style == AppBarStyle::CenterAligned {
                    top.row()?.set_grow(1.0)?;
                    // Centered on the bar, whatever sits beside it.
                    titles.set_absolute(Some(Insets {
                        left: Length::Px(0.0),
                        right: Length::Px(0.0),
                        top: Length::Px(0.0),
                        bottom: Length::Px(0.0),
                    }))?;
                    titles.set_align_items(Some(Align::Center))?;
                    titles.set_justify_content(Some(aegle_ui::Justify::Center))?;
                }
                (typescale::TITLE_LARGE, titles)
            }
            AppBarStyle::Medium | AppBarStyle::Large => {
                let spacer = top.row()?;
                spacer.set_grow(1.0)?;
                let titles = bar.column()?;
                titles.set_gap(0.0)?;
                let (role, height): (TypeStyle, _) = if style == AppBarStyle::Medium {
                    (typescale::HEADLINE_MEDIUM, 48.0)
                } else {
                    (typescale::DISPLAY_SMALL, 56.0)
                };
                titles.set_min_height(height)?;
                titles.set_justify_content(Some(aegle_ui::Justify::End))?;
                titles.set_padding(crate::edges(16.0, 0.0, 16.0, 12.0))?;
                (role, titles)
            }
        };
        let title = Text::new(&titles, role, Role::on_surface, title)?;
        let actions = top.row()?;
        actions.set_gap(0.0)?;
        Ok(Self {
            bar,
            navigation,
            titles,
            actions,
            title,
            style,
        })
    }

    /// Puts a standard icon button, such as a menu or back button, before
    /// the title.
    pub fn navigation(&self, icon: Icon, label: &str) -> Result<IconButton> {
        self.navigation.set_visible(true)?;
        if self.style == AppBarStyle::Small {
            self.titles.set_padding(crate::edges(4.0, 0.0, 12.0, 0.0))?;
        }
        IconButton::new(&self.navigation, IconStyle::Standard, icon, label)
    }

    /// Appends a standard icon button to the trailing actions.
    pub fn action(&self, icon: Icon, label: &str) -> Result<IconButton> {
        IconButton::new(&self.actions, IconStyle::Standard, icon, label)
    }

    /// The title.
    pub fn title(&self) -> &Text {
        &self.title
    }

    /// Adds a subtitle under the title, in label medium (title medium in
    /// the flexible bars).
    pub fn subtitle(&self, text: &str) -> Result<Text> {
        let role = match self.style {
            AppBarStyle::Small | AppBarStyle::CenterAligned => typescale::LABEL_MEDIUM,
            _ => typescale::TITLE_MEDIUM,
        };
        Text::new(&self.titles, role, Role::on_surface_variant, text)
    }

    /// Shows whether content is scrolled under the bar, which fills it with
    /// the surface container.
    pub fn set_scrolled(&self, scrolled: bool) -> Result {
        let kind = if scrolled {
            &TOP_APP_BAR_SCROLLED
        } else {
            &TOP_APP_BAR
        };
        surface::set_kind(&self.bar, kind)
    }
}

/// A bottom app bar: up to four actions and a FAB.
#[derive(Clone)]
pub struct BottomAppBar {
    bar: Container,
    actions: Container,
}

impl std::ops::Deref for BottomAppBar {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.bar
    }
}

impl BottomAppBar {
    /// Appends an 80 dp bar, as wide as its parent allows.
    pub fn new(parent: &Container) -> Result<Self> {
        let control = SurfaceControl::new(&BOTTOM_APP_BAR, Part::Pane, 0.0, 0.0);
        let bar = surface::add(parent, control, 0.0)?;
        bar.set_direction(aegle_ui::Direction::Row)?;
        bar.set_align_self(Some(Align::Stretch))?;
        bar.set_align_items(Some(Align::Center))?;
        bar.set_height(80.0)?;
        bar.set_padding(crate::edges(4.0, 0.0, 16.0, 0.0))?;
        let actions = bar.row()?;
        actions.set_gap(0.0)?;
        actions.set_grow(1.0)?;
        Ok(Self { bar, actions })
    }

    /// Appends a standard icon button.
    pub fn action(&self, icon: Icon, label: &str) -> Result<IconButton> {
        IconButton::new(&self.actions, IconStyle::Standard, icon, label)
    }

    /// Puts a FAB at the end.
    pub fn fab(&self, icon: Icon, label: &str) -> Result<Fab> {
        Fab::new(&self.bar, icon, label)
    }
}
