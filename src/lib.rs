#![no_std]

use gstd::{msg, prelude::*, exec};
use io::*;

static mut GAME: Option<GameState> = None;

#[no_mangle]
extern fn init() {
    let init_msg: PebblesInit = msg::load().expect("Failed to load init message.");
    // Sanity check.
    if init_msg.pebbles_count <= init_msg.max_pebbles_per_turn {
        let _ = msg::reply("Pebbles count must be greater than max pebbles per turn.", 0);
    }

    let random = get_random_u32();
    let first_player = if random % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };
    let mut pebbles_remaining = init_msg.pebbles_count;

    // If the first player is the program, automatically play the first turn.
    if first_player == Player::Program {
        let first_turn_pebbles = get_random_u32() % init_msg.max_pebbles_per_turn;
        // This will not underflow because K < N is enforced during the initialization.
        pebbles_remaining -= first_turn_pebbles;
    }

    unsafe {
        GAME = Some(GameState {
            pebbles_count: init_msg.pebbles_count,
            max_pebbles_per_turn: init_msg.max_pebbles_per_turn,
            pebbles_remaining,
            difficulty: init_msg.difficulty,
            first_player,
            winner: None,
        })
    }

    let _ = msg::reply("Game initiated!", 0).expect("Failed to reply to init message.");
}

#[no_mangle]
extern fn handle() {
    let user_action = msg::load().expect("Failed to load user action.");

    let game = unsafe { GAME.as_mut().expect("Game state not initialized.") };

    match user_action {
        PebblesAction::Turn(pebbles) => {
            // Sanity check.
            if pebbles > game.max_pebbles_per_turn {
                let _ = msg::reply("Pebbles count exceeds the maximum allowed.", 0);
            }
            // If the removed pebbles are greater than the remaining, the user wins.
            if pebbles >= game.pebbles_remaining {
                game.winner = Some(Player::User);
                // Reset the game.
                game.reset();
                let _ = msg::reply(PebblesEvent::Won(Player::User), 0);
            } else {
                if game.difficulty == DifficultyLevel::Easy {
                    // Easy mode: Randomly remove pebbles.
                    let program_pebbles = get_random_u32() % game.max_pebbles_per_turn;
                    // If the removed pebbles are greater than the remaining, the program wins.
                    if program_pebbles >= game.pebbles_remaining {
                        game.winner = Some(Player::Program);
                        // Reset the game.
                        game.reset();
                        let _ = msg::reply(PebblesEvent::Won(Player::Program), 0);
                    } else {
                        game.pebbles_remaining -= program_pebbles;
                        let _ = msg::reply(PebblesEvent::CounterTurn(program_pebbles), 0);
                    }
                }
            }
        },
        _ => {}
    }
}

#[no_mangle]
extern fn state() {}

//----------------------------------- Helper Functions --------------------------------------
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}
