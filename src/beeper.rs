use actually_beep::beep_with_hz_and_millis;

fn play_sound(hz: u32)
{
    let middle_e_hz = hz;
    let a_bit_more_than_a_second_and_a_half_ms = 200;
    beep_with_hz_and_millis(middle_e_hz, a_bit_more_than_a_second_and_a_half_ms).unwrap();
}

pub fn ok_sound()
{
    play_sound(1900);
    play_sound(2600);
}
pub fn error_sound()
{
    play_sound(600);
    play_sound(400);
    play_sound(100);
}




#[cfg(test)]
mod tests
{
    #[test]
    fn test_beep()
    {
        super::play_sound(1900);
        super::play_sound(2600);
        // super::play_sound_2(200);
        // super::play_sound_2(2100);
        // super::play_sound_2(200);
        // super::play_sound_2(2300);
        // super::play_sound_2(200);
        // super::play_sound_2(2500);
        // super::play_sound_2(2400);
        // super::play_sound_2(2300);
        // super::play_sound_2(2200);
        // super::play_sound_2(2100);
        // super::play_sound_2(2000);
        // super::play_sound_2(1900);
    }
    #[test]
    fn test_beep2()
    {
        super::play_sound(600);
        super::play_sound(400);
        super::play_sound(100);
        // super::play_sound_2(200);
        // super::play_sound_2(2100);
        // super::play_sound_2(200);
        // super::play_sound_2(2300);
        // super::play_sound_2(200);
        // super::play_sound_2(2500);
        // super::play_sound_2(2400);
        // super::play_sound_2(2300);
        // super::play_sound_2(2200);
        // super::play_sound_2(2100);
        // super::play_sound_2(2000);
        // super::play_sound_2(1900);
    }
}