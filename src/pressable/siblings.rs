//! How pressable controls act on their siblings: exclusive selection and
//! the standard group's bounce.

use aegle_ui::{NodeId, Result, State};

use super::PressableControl;

/// Deselects the exclusive pressable siblings of `id`.
pub(super) fn select_exclusive(state: &mut State, id: NodeId) -> Result {
    let Some(parent) = state.tree.parent(id)? else {
        return Ok(());
    };
    let siblings: Vec<_> = state.tree.children(parent)?.filter(|&n| n != id).collect();
    for sibling in siblings {
        if let Some(other) = state.control_as::<PressableControl>(sibling)
            && other.exclusive
            && other.selected == Some(true)
        {
            other.selected = Some(false);
            let dirty = aegle_ui::Dirty::PAINT | aegle_ui::Dirty::SEMANTICS;
            state.tree.mark_dirty(sibling, dirty)?;
        }
    }
    Ok(())
}

/// Widens a pressed button of a standard group by 15 % into the
/// neighboring buttons, which give way; or returns all of them.
pub(super) fn bounce(state: &mut State, id: NodeId, pressed: bool) -> Result {
    let Some(parent) = state.tree.parent(id)? else {
        return Ok(());
    };
    let siblings: Vec<_> = state.tree.children(parent)?.collect();
    let index = siblings
        .iter()
        .position(|&n| n == id)
        .expect("a child of its parent");
    let mut grouped = |n: Option<&NodeId>| {
        n.copied().filter(|&n| {
            state
                .control_as::<PressableControl>(n)
                .is_some_and(|c| c.bounce)
        })
    };
    let neighbors = [
        index.checked_sub(1).and_then(|i| grouped(siblings.get(i))),
        grouped(siblings.get(index + 1)),
    ];
    let sides = neighbors.iter().flatten().count();
    let control = state
        .control_as::<PressableControl>(id)
        .expect("a pressable");
    let amount = if pressed && sides > 0 {
        0.15 * control.width / sides as f32
    } else {
        0.0
    };
    control.reach = neighbors.map(|n| if n.is_some() { amount } else { 0.0 });
    let dirty = aegle_ui::Dirty::PAINT;
    state.tree.mark_dirty(id, dirty)?;
    for (side, neighbor) in neighbors.into_iter().enumerate() {
        if let Some(n) = neighbor {
            // The neighbor before gives way at its end, the one after at its start.
            let control = state
                .control_as::<PressableControl>(n)
                .expect("a pressable");
            control.reach[1 - side] = -amount;
            state.tree.mark_dirty(n, dirty)?;
        }
    }
    Ok(())
}
