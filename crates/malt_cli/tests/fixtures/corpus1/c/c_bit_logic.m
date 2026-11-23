function [crc, bits] = c_bit_logic(bytes, mode)
%C_BIT_LOGIC Simple checksum and bit-field helpers for a packet decoder.
%   [CRC, BITS] = C_BIT_LOGIC(BYTES, MODE) computes a 16-bit checksum over
%   BYTES. MODE selects the polynomial ('ccitt' or 'modbus').

if nargin < 2
    mode = 'ccitt';
end

% Lookup table of field widths, keyed by packet type
fieldWidths = containers.Map();
fieldWidths('header') = 8;
fieldWidths('payload') = 16;
fieldWidths('footer') = 4;

% Checksum loop with unsigned arithmetic
crc = uint32(65535);
for k = 1:numel(bytes)
    crc = bitxor(crc, uint32(bytes(k)));
    for bit = 1:8
        if bitand(crc, uint32(1))
            crc = bitshift(crc, -1);
            crc = bitxor(crc, uint32(40961));
        else
            crc = bitshift(crc, -1);
        end
    end
end
crc = bitand(crc, uint32(65535));

% Pack a 32-bit word and read back the bytes
word = typecast(single(1.5), 'uint32');
bitsText = dec2bin(word, 32);
bits = bitsText == '1';
recovered = bin2dec(bitsText);
assert(recovered == double(word), 'Round trip of the float word failed');

% Split a header byte into its nibbles
hdr = uint8(bytes(1));
hi = bitshift(hdr, -4);
lo = bitand(hdr, uint8(15));
fprintf('header nibbles: %d %d\n', hi, lo);

% Wrap-around counter with mod and rem
counter = 0;
for n = 1:numel(bytes)
    counter = mod(counter + double(bytes(n)), 256);
end
remainder = rem(-7, 3);   % sign follows the dividend here

% Map each command code to a human-readable action
actions = containers.Map({'01', '02', '03'}, {'start', 'stop', 'reset'});
cmdCode = sprintf('%02d', bytes(2));
if isKey(actions, cmdCode)
    cmdName = actions(cmdCode);
else
    cmdName = 'unknown';
end

% Switch over the numeric packet type
packetType = double(bytes(3));
switch packetType
    case 1
        kind = 'data';
    case {2, 3}
        kind = 'control';
    case 4
        kind = 'status';
    otherwise
        kind = 'reserved';
end

fprintf('crc=%d counter=%d rem=%d cmd=%s kind=%s\n', ...
    crc, counter, remainder, cmdName, kind);
fprintf('width of payload field: %d\n', fieldWidths('payload'));

% Build a flag mask from a list of bit positions
positions = [0 3 5 7];
flagMask = uint8(0);
for p = positions
    flagMask = bitor(flagMask, bitshift(uint8(1), p));
end
bits = [bits; dec2bin(flagMask, 8) == '1'];
end
