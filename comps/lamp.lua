NUM_OF_INS = 1
NUM_OF_OUTS = 0
MEMORY_SIZE = 1

WIDTH = 2
HEIGHT = 2

INPUT_POSITIONS = {7}
OUTPUT_POSITIONS = {}

ON = false

function Calculate(inputs)
    ON = inputs[1]
    return {inputs[1]}
end

function Draw(buffer)
    buffer:set_all(0, 255, 255, 255)
    -- rust signature: (u32, u32, u32, u32, (u8, u8, u8, u8))
    if ON then
        -- draw green rect
        buffer:add_rect(10, 10, 20, 20, 0, 255, 0, 255)
    else
        -- draw red rect
        buffer:add_rect(10, 10, 20, 20, 255, 0, 0, 255)
    end
end
