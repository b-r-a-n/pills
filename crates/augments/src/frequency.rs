use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Frequency {
    pub amount: i32,
}

pub(crate) fn apply(
    augments: Query<(&Frequency, &InBoard), Added<InBoard>>,
    mut boards: Query<&mut BoardConfig>,
) {
    for (augment, board_id) in &augments {
        if let Ok(mut config) = boards.get_mut(**board_id) {
            let new_amount = config.max_viruses as i32 + augment.amount;
            let (r, c) = config.board_size;
            let max_amount = r as i32 * (c as i32 - 3);
            if new_amount > 0 && new_amount < max_amount {
                config.max_viruses = new_amount as usize;
            }

        }
    }
}
