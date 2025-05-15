%builtins output range_check bitwise

from src.sha256 import (
    sha256,
    finalize_sha256,
)
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.uint256 import uint256_eq

// ref: https://github.com/cartridge-gg/cairo-sha256/blob/main/tests/test_sha256.cairo#L13
func main{output_ptr: felt*, range_check_ptr, bitwise_ptr: BitwiseBuiltin*}() {
    alloc_locals;

    let (input_text) = alloc();

    // Declare a local variable to hold the number of bytes
    local num_bytes;

    %{
    import math

    # Retrieve the input text from the program input
    text = program_input['text']

    # Convert the text to bytes
    text_bytes = text.encode('utf-8')

    # Store the number of segments in a variable for use in Cairo
    ids.num_bytes = len(text_bytes)

    # Calculate the number of 4-byte segments needed
    num_segments = math.ceil(len(text_bytes) / 4)

    # Pad the byte array to make its length a multiple of 4
    padded_bytes = text_bytes.ljust(num_segments * 4, b'\x00')

    # Split the byte array into 4-byte segments and convert each to a felt
    segments = [int.from_bytes(padded_bytes[i:i+4], byteorder='big') for i in range(0, len(padded_bytes), 4)]

    # Write each segment into memory starting at the 'input_text' pointer
    for i, segment in enumerate(segments):
        memory[ids.input_text + i] = segment
    
    %}

    // Assign the number of bytes from the hint
    let num_bytes = num_bytes;

    let (local sha256_ptr: felt*) = alloc();
    let sha256_ptr_start = sha256_ptr;
    let (hash) = sha256{sha256_ptr=sha256_ptr}(input_text, num_bytes);
    finalize_sha256(sha256_ptr_start=sha256_ptr_start, sha256_ptr_end=sha256_ptr);

    return ();
}