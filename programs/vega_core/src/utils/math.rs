
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
    _deposit_amount: u64,
    _start: u64,
    _end: u64,
    _before_lp: u64,
    _after_lp: u64,
) -> u64 {
    let duration: u64 = _end.checked_sub(_start).unwrap();
    let standard_time: u64 = 2;
    let reward_rate: u64 = 3;
    let reward_amount_per_2_sec: u64 = _deposit_amount
        .checked_mul(reward_rate)
        .unwrap()
        .checked_div(10_000_000)
        .unwrap();

    let mut reward: u64 = duration
        .checked_div(standard_time)
        .unwrap()
        .checked_mul(reward_amount_per_2_sec)
        .unwrap();

    reward = _before_lp.checked_add(reward).unwrap();
    let (pr, is_pl) = calc_pl(_before_lp, _after_lp, 0);
    
    calc_trade_out_lp_amount(reward, pr, is_pl)
}

pub fn calc_pl(_entry: u64, _out: u64, _way: u8) -> (u64, bool) {
    
    if _entry <= _out {
        let a = _entry.checked_mul(100).unwrap();
        let b = _out.checked_sub(_entry).unwrap().checked_mul(100).unwrap();
        let pr = b.checked_mul(100).unwrap().checked_div(a).unwrap();
        let pl = if _way == 0 {true} else {false};
        return (pr, pl );

    } else {
        let a = _entry.checked_mul(100).unwrap();
        let b = _entry.checked_sub(_out).unwrap().checked_mul(100).unwrap();
        let pr = b.checked_mul(100).unwrap().checked_div(a).unwrap();
        let pl = if _way == 0 {false} else {true};
        return (pr, pl);

    }
}

pub fn calc_trade_out_lp_amount (_current_lp : u64, _percentage :u64, _pl : bool) -> u64 {
    if _pl {
        let a = _percentage.checked_add(100).unwrap();
        return _current_lp.checked_mul(a).unwrap().checked_div(100).unwrap();
    } else {
        if _percentage >= 100 {
            return 0;
        }else {
            let b : u64 = 100;
            let c = b.checked_sub(_percentage).unwrap();
            return _current_lp.checked_mul(c).unwrap().checked_div(100).unwrap();
        }

    }

}
