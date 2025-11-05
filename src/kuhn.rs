use std::{fmt, vec};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Check,
    Bet,
    Call,
    Fold,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Action::Check => "Check",
            Action::Bet => "Bet",
            Action::Call => "Call",
            Action::Fold => "Fold",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct History {
    actions: Vec<Action>,
}

impl Hash for History {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for action in &self.actions {
            action.hash(state);
        }
    }
}

impl History {
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    pub fn add(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn to_string(&self) -> String {
        self.actions
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub history: History,
    pub current_player: Player,
    pub terminal: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            history: History::new(),
            current_player: Player::Player1,
            terminal: false,
        }
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        use Action::*;
        if self.terminal {
            return vec![];
        }

        match self.history.actions.as_slice() {
            [] => vec![Check, Bet],

            // P1. Check -> P2. Check or Bet
            [Check] => vec![Check, Bet],

            // P1. Check -> P2. Bet -> P1. Call or Fold
            [Check, Bet] => vec![Call, Fold],

            // P1. Bet -> P2. Call or Fold
            [Bet] => vec![Call, Fold],

            _ => vec![],
        }
    }

    pub fn next_state(&self, action: Action) -> GameState {
        let mut new_history: History = self.history.clone();
        new_history.add(action);

        let mut next: GameState = GameState {
            history: new_history,
            current_player: self.current_player.other(),
            terminal: false,
        };

        use Action::*;
        match next.history.actions.as_slice() {
            [Check, Check] | [Bet, Fold] | [Check, Bet, Fold] | [Bet, Call] | [Check, Bet, Call] => {
                next.terminal = true;
            }
            _ => {}
        }

        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        assert_eq!(Action::Check.to_string(), "Check");
        assert_eq!(Action::Bet.to_string(), "Bet");
        assert_eq!(Action::Call.to_string(), "Call");
        assert_eq!(Action::Fold.to_string(), "Fold");
    }

    #[test]
    fn test_player_other() {
        assert_eq!(Player::Player1.other(), Player::Player2);
        assert_eq!(Player::Player2.other(), Player::Player1);
    }

    #[test]
    fn test_history_add_to_string() {
        let state = GameState::new();
        assert_eq!(state.legal_actions(), vec![Action::Check, Action::Bet]);

        let state = state.next_state(Action::Check);
        assert_eq!(state.legal_actions(), vec![Action::Check, Action::Bet]);

        let state = state.next_state(Action::Bet);
        assert_eq!(state.legal_actions(), vec![Action::Call, Action::Fold]);
    }

    #[test]
    fn test_game_state_terminal() {
        let state = GameState::new()
            .next_state(Action::Check)
            .next_state(Action::Check);
        assert!(state.terminal);

        let state = GameState::new()
            .next_state(Action::Bet)
            .next_state(Action::Call);
        assert!(state.terminal);

    }
}
