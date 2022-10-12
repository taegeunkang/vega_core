
pub fn sqrt(y: u64) -> u64 {
    let mut _z: u64 = 0;

    if y > 3 {
        _z = y;
        let mut x = y.checked_div(2).unwrap().checked_add(1).unwrap();

        loop {
            _z = x;
            x = y
                .checked_div(x)
                .unwrap()
                .checked_add(x)
                .unwrap()
                .checked_div(2)
                .unwrap();

            if x >= _z {
                break;
            }
        }
    } else if y != 0 {
        _z = 1;
    }

    _z
}

pub fn calc_fee_amount(amount: u64, fee_rate: u8) -> u64 {
    amount
        .checked_mul(fee_rate as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
}

pub fn calc_lp_amount(pool_vault_amount: u64, pool_lp_supply: u64, amount: u64) -> u64 {
    let pool_amount: u64 = pool_vault_amount;

    let total_supply: u64 = pool_lp_supply;
    let mut _liquidity: u64 = 0;
   
    if total_supply == 0 {
        _liquidity = sqrt(amount).checked_sub(1000).unwrap();
        //1000개 기본 락
    } else {
        _liquidity = amount
            .checked_mul(total_supply)
            .unwrap()
            .checked_div(pool_amount)
            .unwrap();
    }

    _liquidity
}

pub fn calc_reward_percent(
    deposit_amount: u64,
    start: u64,
    end: u64,
    before_lp: u64,
    after_lp: u64,
) -> u64 {
    let duration: u64 = end.checked_sub(start).unwrap();
    let standard_time: u64 = 2;
    let reward_rate: u64 = 3;
    let reward_amount_per_2_sec: u64 = deposit_amount
        .checked_mul(reward_rate)
        .unwrap()
        .checked_div(10_000_000)
        .unwrap();

    let mut reward: u64 = duration
        .checked_div(standard_time)
        .unwrap()
        .checked_mul(reward_amount_per_2_sec)
        .unwrap();

    reward = deposit_amount.checked_add(reward).unwrap();
    let (pr, is_pl) = calc_pl(before_lp, after_lp, 0);
    return calc_withdraw_amount(reward, pr, is_pl);
}
pub fn calc_withdraw_amount(amount: u64, pr: u64, is_pl: bool) -> u64 {
    if is_pl {
        let a: u64 = pr.checked_add(100).unwrap();
        return a.checked_mul(amount).unwrap().checked_div(100).unwrap();
    } else {
        if pr >= 100 {
            return 0;
        } else {
            let b: u64 = 100;
            let c: u64 = b.checked_sub(pr).unwrap();
            return amount.checked_mul(c).unwrap().checked_div(100).unwrap();
        }
    }
}

pub fn calc_pl(entry: u64, out: u64, way: u8) -> (u64, bool) {
    if entry <= out {
        let a = entry.checked_mul(10000).unwrap();
        let b = out.checked_sub(entry).unwrap().checked_mul(10000).unwrap();
        let pr = b.checked_mul(10000).unwrap().checked_div(a).unwrap();
        let pl = if way == 0 { true } else { false };
        return (pr, pl);
    } else {
        let a = entry.checked_mul(100).unwrap();
        let b = entry.checked_sub(out).unwrap().checked_mul(100).unwrap();
        let pr = b.checked_mul(100).unwrap().checked_div(a).unwrap();
        let pl = if way == 0 { false } else { true };
        return (pr, pl);
    }
}

pub fn calc_trade_out_lp_amount(current_lp: u64, percentage: u64, pl: bool) -> u64 {
    if pl {
        let a = percentage.checked_add(10000).unwrap();
        return current_lp
            .checked_mul(a)
            .unwrap()
            .checked_div(10000)
            .unwrap();
    } else {
        if percentage >= 100 {
            return 0;
        } else {
            let b: u64 = 10000;
            let c = b.checked_sub(percentage).unwrap();
            return current_lp
                .checked_mul(c)
                .unwrap()
                .checked_div(10000)
                .unwrap();
        }
    }
}
