use std::collections::HashMap;
use crate::kuhn::{Action, GameState};

/// CFRノード: 各情報集合での累積リグレットと戦略を管理
#[derive(Debug, Clone)]
pub struct CFRNode {
    /// 各アクションの累積リグレット
    pub regret_sum: HashMap<Action, f64>,
    /// 各アクションの累積戦略（平均戦略の計算用）
    pub strategy_sum: HashMap<Action, f64>,
    /// この情報集合で取りうるアクション
    pub actions: Vec<Action>,
}

impl CFRNode {
    /// 新しいCFRノードを作成
    pub fn new(actions: Vec<Action>) -> Self {
        let mut regret_sum = HashMap::new();
        let mut strategy_sum = HashMap::new();
        
        for action in &actions {
            regret_sum.insert(*action, 0.0);
            strategy_sum.insert(*action, 0.0);
        }
        
        Self {
            regret_sum,
            strategy_sum,
            actions,
        }
    }

    /// リグレットマッチングで現在の戦略を計算
    /// 正のリグレットを持つアクションに比例した確率を割り当てる
    pub fn get_strategy(&self) -> HashMap<Action, f64> {
        let mut strategy = HashMap::new();
        let mut normalizing_sum = 0.0;

        // 正のリグレットの合計を計算
        for action in &self.actions {
            let regret = *self.regret_sum.get(action).unwrap();
            let positive_regret = regret.max(0.0);
            normalizing_sum += positive_regret;
        }

        // 正のリグレットがある場合は比例配分、ない場合は均等配分
        for action in &self.actions {
            if normalizing_sum > 0.0 {
                let regret = *self.regret_sum.get(action).unwrap();
                let positive_regret = regret.max(0.0);
                strategy.insert(*action, positive_regret / normalizing_sum);
            } else {
                // 全てのリグレットが非正の場合は均等戦略
                strategy.insert(*action, 1.0 / self.actions.len() as f64);
            }
        }

        strategy
    }

    /// 累積戦略から平均戦略を計算
    /// これがGTO戦略に収束する
    pub fn get_average_strategy(&self) -> HashMap<Action, f64> {
        let mut avg_strategy = HashMap::new();
        let mut normalizing_sum = 0.0;

        // 累積戦略の合計を計算
        for action in &self.actions {
            normalizing_sum += *self.strategy_sum.get(action).unwrap();
        }

        // 正規化して平均戦略を計算
        for action in &self.actions {
            if normalizing_sum > 0.0 {
                let sum = *self.strategy_sum.get(action).unwrap();
                avg_strategy.insert(*action, sum / normalizing_sum);
            } else {
                // 累積がない場合は均等戦略
                avg_strategy.insert(*action, 1.0 / self.actions.len() as f64);
            }
        }

        avg_strategy
    }
}

/// 情報集合を管理するマップ
/// キー: "カード-履歴" の形式（例: "J-Check-Bet", "Q-Bet"）
/// 値: CFRNode
pub type InfoSetMap = HashMap<String, CFRNode>;

/// 情報集合のキーを生成
/// card: プレイヤーのカード ('J', 'Q', 'K')
/// history: アクション履歴の文字列
pub fn make_info_set_key(card: char, history: &str) -> String {
    if history.is_empty() {
        format!("{}", card)
    } else {
        format!("{}-{}", card, history)
    }
}

/// Kuhn Pokerのカード (J=11, Q=12, K=13)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Jack = 11,
    Queen = 12,
    King = 13,
}

impl Card {
    /// カードを文字に変換
    pub fn to_char(&self) -> char {
        match self {
            Card::Jack => 'J',
            Card::Queen => 'Q',
            Card::King => 'K',
        }
    }

    /// カードの強さを数値で取得
    pub fn rank(&self) -> i32 {
        *self as i32
    }
}

/// 2人のプレイヤーにカードを配る全ての組み合わせを返す
/// Kuhn Pokerでは3枚(J,Q,K)から2枚を選んで配る
pub fn deal_cards() -> Vec<(Card, Card)> {
    use Card::*;
    vec![
        (Jack, Queen),
        (Jack, King),
        (Queen, Jack),
        (Queen, King),
        (King, Jack),
        (King, Queen),
    ]
}

/// 終端状態での報酬を計算
/// player: 報酬を計算するプレイヤー
/// card1: プレイヤー1のカード
/// card2: プレイヤー2のカード
/// history: アクション履歴（文字列）
/// 返り値: プレイヤー1から見た報酬（プレイヤー2は符号を反転）
pub fn get_payoff(card1: Card, card2: Card, history: &str) -> i32 {
    // 履歴を解析（簡易的に文字列で判定）
    match history {
        // Check-Check: ショーダウン、ポット=2
        "Check-Check" => {
            if card1.rank() > card2.rank() { 1 } else { -1 }
        }
        
        // Bet-Fold: P1がBet、P2がFold → P1が1チップ獲得
        "Bet-Fold" => 1,
        
        // Check-Bet-Fold: P1 Check、P2 Bet、P1 Fold → P1が1チップ失う
        "Check-Bet-Fold" => -1,
        
        // Bet-Call: P1がBet、P2がCall → ショーダウン、ポット=4
        "Bet-Call" => {
            if card1.rank() > card2.rank() { 2 } else { -2 }
        }
        
        // Check-Bet-Call: P1 Check、P2 Bet、P1 Call → ショーダウン、ポット=4
        "Check-Bet-Call" => {
            if card1.rank() > card2.rank() { 2 } else { -2 }
        }
        
        _ => 0, // それ以外（終端でない場合など）
    }
}

/// CFRアルゴリズムの本体
/// state: 現在のゲーム状態
/// card1: プレイヤー1のカード
/// card2: プレイヤー2のカード
/// info_sets: 情報集合のマップ（学習データの蓄積先）
/// 返り値: プレイヤー1から見た期待値
pub fn cfr(
    state: &GameState,
    card1: Card,
    card2: Card,
    info_sets: &mut InfoSetMap,
) -> f64 {
    let history = state.history.to_string();
    
    // 終端状態なら報酬を返す
    if state.terminal {
        return get_payoff(card1, card2, &history) as f64;
    }
    
    // 現在のプレイヤーのカードと情報集合キーを取得
    let card = match state.current_player {
        crate::kuhn::Player::Player1 => card1,
        crate::kuhn::Player::Player2 => card2,
    };
    let info_set_key = make_info_set_key(card.to_char(), &history);
    
    // 合法手を取得
    let actions = state.legal_actions();
    
    // 情報集合が存在しなければ作成
    if !info_sets.contains_key(&info_set_key) {
        info_sets.insert(info_set_key.clone(), CFRNode::new(actions.clone()));
    }
    
    // 現在の戦略を取得
    let node = info_sets.get(&info_set_key).unwrap();
    let strategy = node.get_strategy();
    
    // 各アクションの価値を計算
    let mut action_values: HashMap<Action, f64> = HashMap::new();
    let mut node_value = 0.0;
    
    for action in &actions {
        let next_state = state.next_state(*action);
        let action_value = cfr(&next_state, card1, card2, info_sets);
        
        // プレイヤー2の価値は符号を反転
        let value = match state.current_player {
            crate::kuhn::Player::Player1 => action_value,
            crate::kuhn::Player::Player2 => -action_value,
        };
        
        action_values.insert(*action, value);
        node_value += strategy.get(action).unwrap() * value;
    }
    
    // リグレットと戦略を更新
    let node = info_sets.get_mut(&info_set_key).unwrap();
    for action in &actions {
        let action_value = *action_values.get(action).unwrap();
        let regret = action_value - node_value;
        
        // リグレット累積
        *node.regret_sum.get_mut(action).unwrap() += regret;
        
        // 戦略累積（現在の戦略を加算）
        *node.strategy_sum.get_mut(action).unwrap() += *strategy.get(action).unwrap();
    }
    
    // プレイヤー2の場合は符号を反転して返す
    match state.current_player {
        crate::kuhn::Player::Player1 => node_value,
        crate::kuhn::Player::Player2 => -node_value,
    }
}

/// Kuhn PokerのGTO戦略をトレーニング
/// iterations: イテレーション回数
/// 返り値: 学習済みの情報集合マップ
pub fn train(iterations: usize) -> InfoSetMap {
    let mut info_sets = InfoSetMap::new();
    
    print!("Progress: [");
    
    for i in 0..iterations {
        // 全てのカード配布パターンについてCFRを実行
        for (card1, card2) in deal_cards() {
            let state = GameState::new();
            cfr(&state, card1, card2, &mut info_sets);
        }
        
        // 進捗表示（10%ごと）
        if iterations >= 10 && (i + 1) % (iterations / 10) == 0 {
            print!("■");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    }
    
    println!("] 100%");
    
    info_sets
}

/// GTO戦略を見やすく表示
pub fn print_strategy(info_sets: &InfoSetMap) {
    println!("\n=== Kuhn Poker GTO Strategy ===\n");
    
    // 情報集合をソートして表示
    let mut keys: Vec<&String> = info_sets.keys().collect();
    keys.sort();
    
    for key in keys {
        let node = info_sets.get(key).unwrap();
        let avg_strategy = node.get_average_strategy();
        
        println!("Information Set: {}", key);
        
        // アクションをソート
        let mut actions: Vec<&Action> = avg_strategy.keys().collect();
        actions.sort_by_key(|a| format!("{:?}", a));
        
        for action in actions {
            let prob = avg_strategy.get(action).unwrap();
            println!("  {:?}: {:.2}%", action, prob * 100.0);
        }
        println!();
    }
}

/// メイン実行関数
pub fn main() {
    println!("Training Kuhn Poker GTO strategy...\n");
    
    // 10,000回イテレーション
    let iterations = 10_000;
    let info_sets = train(iterations);
    
    println!("\nTraining complete!");
    println!("Total information sets: {}", info_sets.len());
    
    // GTO戦略を表示
    print_strategy(&info_sets);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfr_node_creation() {
        let actions = vec![Action::Check, Action::Bet];
        let node = CFRNode::new(actions.clone());
        
        assert_eq!(node.actions.len(), 2);
        assert_eq!(*node.regret_sum.get(&Action::Check).unwrap(), 0.0);
        assert_eq!(*node.regret_sum.get(&Action::Bet).unwrap(), 0.0);
        assert_eq!(*node.strategy_sum.get(&Action::Check).unwrap(), 0.0);
        assert_eq!(*node.strategy_sum.get(&Action::Bet).unwrap(), 0.0);
    }

    #[test]
    fn test_info_set_key_generation() {
        assert_eq!(make_info_set_key('J', ""), "J");
        assert_eq!(make_info_set_key('Q', "Check"), "Q-Check");
        assert_eq!(make_info_set_key('K', "Bet-Call"), "K-Bet-Call");
    }

    #[test]
    fn test_get_strategy_uniform() {
        // リグレットが全て0の場合、均等戦略になる
        let actions = vec![Action::Check, Action::Bet];
        let node = CFRNode::new(actions);
        let strategy = node.get_strategy();
        
        assert_eq!(*strategy.get(&Action::Check).unwrap(), 0.5);
        assert_eq!(*strategy.get(&Action::Bet).unwrap(), 0.5);
    }

    #[test]
    fn test_get_strategy_with_regret() {
        // リグレットがある場合、それに比例した戦略になる
        let actions = vec![Action::Check, Action::Bet];
        let mut node = CFRNode::new(actions);
        
        // Checkのリグレットを3.0、Betのリグレットを1.0に設定
        node.regret_sum.insert(Action::Check, 3.0);
        node.regret_sum.insert(Action::Bet, 1.0);
        
        let strategy = node.get_strategy();
        
        // 3:1の比率で戦略が割り当てられる
        assert_eq!(*strategy.get(&Action::Check).unwrap(), 0.75);
        assert_eq!(*strategy.get(&Action::Bet).unwrap(), 0.25);
    }

    #[test]
    fn test_get_average_strategy() {
        // 累積戦略から平均戦略を計算
        let actions = vec![Action::Check, Action::Bet];
        let mut node = CFRNode::new(actions);
        
        // 累積戦略を設定（例: Checkが60回、Betが40回選択された）
        node.strategy_sum.insert(Action::Check, 60.0);
        node.strategy_sum.insert(Action::Bet, 40.0);
        
        let avg_strategy = node.get_average_strategy();
        
        assert_eq!(*avg_strategy.get(&Action::Check).unwrap(), 0.6);
        assert_eq!(*avg_strategy.get(&Action::Bet).unwrap(), 0.4);
    }

    #[test]
    fn test_card_rank() {
        assert_eq!(Card::Jack.rank(), 11);
        assert_eq!(Card::Queen.rank(), 12);
        assert_eq!(Card::King.rank(), 13);
        assert!(Card::King.rank() > Card::Queen.rank());
    }

    #[test]
    fn test_card_to_char() {
        assert_eq!(Card::Jack.to_char(), 'J');
        assert_eq!(Card::Queen.to_char(), 'Q');
        assert_eq!(Card::King.to_char(), 'K');
    }

    #[test]
    fn test_deal_cards() {
        let deals = deal_cards();
        assert_eq!(deals.len(), 6); // 3枚から2枚選ぶ順列 = 3*2 = 6
        
        // 最初の配布がJack-Queenであることを確認
        assert_eq!(deals[0], (Card::Jack, Card::Queen));
    }

    #[test]
    fn test_payoff_check_check() {
        // Check-Check: ショーダウン
        assert_eq!(get_payoff(Card::King, Card::Queen, "Check-Check"), 1);  // Kが勝つ
        assert_eq!(get_payoff(Card::Jack, Card::Queen, "Check-Check"), -1); // Jが負ける
    }

    #[test]
    fn test_payoff_bet_fold() {
        // Bet-Fold: P1が1チップ獲得
        assert_eq!(get_payoff(Card::Jack, Card::King, "Bet-Fold"), 1);
    }

    #[test]
    fn test_payoff_bet_call() {
        // Bet-Call: ショーダウン、ポット=4
        assert_eq!(get_payoff(Card::King, Card::Jack, "Bet-Call"), 2);   // Kが勝つ
        assert_eq!(get_payoff(Card::Jack, Card::King, "Bet-Call"), -2);  // Jが負ける
    }

    #[test]
    fn test_payoff_check_bet_fold() {
        // Check-Bet-Fold: P1が1チップ失う
        assert_eq!(get_payoff(Card::Queen, Card::King, "Check-Bet-Fold"), -1);
    }

    #[test]
    fn test_payoff_check_bet_call() {
        // Check-Bet-Call: ショーダウン、ポット=4
        assert_eq!(get_payoff(Card::King, Card::Queen, "Check-Bet-Call"), 2);
        assert_eq!(get_payoff(Card::Jack, Card::Queen, "Check-Bet-Call"), -2);
    }

    #[test]
    fn test_cfr_single_iteration() {
        // CFRを1回実行して、情報集合が作成されることを確認
        let state = GameState::new();
        let mut info_sets = InfoSetMap::new();
        
        // J vs Qでゲームを実行
        let _value = cfr(&state, Card::Jack, Card::Queen, &mut info_sets);
        
        // 情報集合が作成されているはず
        assert!(info_sets.len() > 0);
        
        // 初手の情報集合（J）が存在するはず
        assert!(info_sets.contains_key("J"));
    }

    #[test]
    fn test_cfr_multiple_iterations() {
        // 複数回CFRを実行して戦略が更新されることを確認
        let mut info_sets = InfoSetMap::new();
        
        // 10回イテレーション
        for _ in 0..10 {
            for (card1, card2) in deal_cards() {
                let state = GameState::new();
                cfr(&state, card1, card2, &mut info_sets);
            }
        }
        
        // 情報集合が複数作成されているはず
        assert!(info_sets.len() > 0);
        
        // 累積戦略が更新されているはず（0より大きい）
        let j_node = info_sets.get("J").unwrap();
        let total_strategy: f64 = j_node.strategy_sum.values().sum();
        assert!(total_strategy > 0.0);
    }
}

