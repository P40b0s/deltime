use actually_beep::beep_with_hz_and_millis;
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

fn play_sound()
{
    // set up the audio context with optimized settings for your hardware
    let context = AudioContext::default();

    // for background music, read from local file
    let file = std::fs::File::open("sounds/n1.wav").unwrap();
    let buffer = context.decode_audio_data_sync(file).unwrap();

    // setup an AudioBufferSourceNode
    let mut src = context.create_buffer_source();
    src.set_buffer(buffer);
    src.set_loop(true);

    // create a biquad filter
    let biquad = context.create_biquad_filter();
    biquad.frequency().set_value(125.);

    // connect the audio nodes
    src.connect(&biquad);
    biquad.connect(&context.destination());

    // play the buffer
    src.start();

    // enjoy listening
    std::thread::sleep(std::time::Duration::from_secs(4))
}

fn play_sound_2()
{
    let middle_e_hz = 200;
  let a_bit_more_than_a_second_and_a_half_ms = 600;

  beep_with_hz_and_millis(middle_e_hz, a_bit_more_than_a_second_and_a_half_ms).unwrap();
}



#[cfg(test)]
mod tests
{
    use std::{arch::asm, time::Duration};
    #[test]
    fn test_beep_2()
    {
        //super::play_sound();
        super::play_sound_2();
    }
    // #[test]
    // fn test_beep()
    // {
    //     //let mut s = super::Speaker::new();
    //     //s.beep(1000, 10);
    //     unsafe {
    //             // Отправляем команду "set tune"
    //             asm!(
    //                 "mov al, 0xB6",
    //                 "out 0x43, al",
    //             );

    //             // Небольшая задержка для завершения операции ввода-вывода
    //             asm!(
    //                 "mov eax, 0x1000",
    //                 "2:",
    //                 "sub eax, 1",
    //                 "cmp eax, 0",
    //                 "jne 1b",
    //             );

    //             // Устанавливаем частоту 220 Гц (0x152F == 1193180 / 220)
    //             asm!(
    //                 "mov al, 0x2F",
    //                 "out 0x42, al",
    //             );

    //             // Небольшая задержка
    //             asm!(
    //                 "mov eax, 0x1000",
    //                 "2:",
    //                 "sub eax, 1",
    //                 "cmp eax, 0",
    //                 "jne 1b",
    //             );

    //             asm!(
    //                 "mov al, 0x15",
    //                 "out 0x42, al",
    //             );

    //             // Небольшая задержка
    //             asm!(
    //                 "mov eax, 0x1000",
    //                 "2:",
    //                 "sub eax, 1",
    //                 "cmp eax, 0",
    //                 "jne 1b",
    //             );

    //             // Включаем динамик
    //             asm!(
    //                 "in al, 0x61",
    //                 "mov ah, al",
    //                 "or al, 0x3",
    //                 "out 0x61, al",
    //             );

    //             // Задержка около 1 секунды
    //             asm!(
    //                 "mov eax, 0x30000000",
    //                 "2:",
    //                 "sub eax, 1",
    //                 "cmp eax, 0",
    //                 "jne 1b",
    //             );

    //             // Выключаем динамик
    //             asm!(
    //                 "mov al, ah",
    //                 "and al, 0xFC",
    //                 "out 0x61, al",
    //             );
    //     }
    // }
}