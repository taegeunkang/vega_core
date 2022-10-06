pub fn sqrt(y: u64) -> u64 {
    let mut z: u64 = 0;

    if y > 3 {
        z = y;
        let mut x = y.checked_div(2).unwrap().checked_add(1).unwrap();

        loop {
            z = x;
            x = y
                .checked_div(x)
                .unwrap()
                .checked_add(x)
                .unwrap()
                .checked_div(2)
                .unwrap();

            if x >= z {
                break;
            }
        }
    } else if y != 0 {
        z = 1;
    }

    z
}

pub fn min(_a: u64, _b: u64) -> u64 {
    let mn = if _a > _b { _b } else { _a };
    mn
}


pub fn fee_amount(_amount : u64, _fee_rate : u16) -> u64 {
    _amount.checked_mul(_fee_rate as u64).unwrap().checked_div(10_000).unwrap()
}