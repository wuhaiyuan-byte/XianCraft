#[cfg(test)]
mod tests {
    use crate::world::world_player::Player;

    #[test]
    fn test_realm_and_level_up_system() {
        // 1. Create a new player
        let mut player = Player::new_character(1, "测试员".to_string(), "凡根".to_string());
        
        // 2. Verify initial stats (level 1, realm 1)
        assert_eq!(player.realm_level, 1);
        assert_eq!(player.realm_sub_level, 1);
        assert_eq!(player.exp, 0);
        
        let initial_hp = player.hp_max;
        let initial_atk = player.atk;
        // Formula: hp = con*10 + 30*1 = 16*10 + 30 = 190
        // Formula: atk = 2*1 = 2
        assert_eq!(initial_hp, 190);
        assert_eq!(initial_atk, 2);

        // 3. Add enough exp to trigger a level up to level 2 (requires 100 exp)
        // 4. Check if realm_sub_level is 2 and vitals (HP, ATK) are updated
        // 7. Verify overflow exp is handled correctly (150 - 100 = 50)
        let msg = player.add_exp(150);
        
        assert_eq!(player.realm_sub_level, 2);
        assert_eq!(player.exp, 50); 
        assert_eq!(player.hp_max, 16 * 10 + 30 * 2); // 220
        assert_eq!(player.atk, 2 * 2); // 4
        assert!(msg.contains("炼气第2层"));

        // 5. Add enough exp to reach level 9
        // Levels and costs:
        // 2->3: 400
        // 3->4: 900
        // 4->5: 1600
        // 5->6: 2500
        // 6->7: 3600
        // 7->8: 4900
        // 8->9: 6400
        let exp_to_9 = 400 + 900 + 1600 + 2500 + 3600 + 4900 + 6400;
        player.add_exp(exp_to_9);
        
        assert_eq!(player.realm_sub_level, 9);
        assert_eq!(player.hp_max, 16 * 10 + 30 * 9); // 430
        assert_eq!(player.atk, 2 * 9); // 18

        // 6. Check if check_promotion returns the bottleneck message
        // Current exp is 50. Level 9 needs 8100 to be "full" for promotion check.
        player.add_exp(8050); 
        assert_eq!(player.exp, 8100);
        
        let promo_msg = player.check_promotion();
        assert!(promo_msg.contains("筑基丹"));
        assert!(promo_msg.contains("瓶颈"));
    }
}