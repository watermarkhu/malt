function out = bits_checksum(data, mode)
% BITS_CHECKSUM Small checksum utilities used by a packet framer.
%   out = BITS_CHECKSUM(data, 'xor') returns the XOR of all bytes.
%   out = BITS_CHECKSUM(data, 'fletcher') returns a 16-bit Fletcher sum.
%   out = BITS_CHECKSUM(data) uses 'xor'.

if nargin < 2 || isempty(mode)
    mode = 'xor';
end
data = uint8(data(:));

persistent callCount
if isempty(callCount)
    callCount = 0;
end
callCount = callCount + 1;

global CHECKSUM_TRACE
if ~isempty(CHECKSUM_TRACE) && CHECKSUM_TRACE
    fprintf('call %d, mode %s, %d bytes\n', callCount, mode, numel(data));
end

switch lower(mode)
    case 'xor'
        out = uint8(0);
        for i = 1:numel(data)
            out = bitxor(out, data(i));
        end
    case 'fletcher'
        s1 = uint16(0);
        s2 = uint16(0);
        for i = 1:numel(data)
            s1 = mod(s1 + uint16(data(i)), 255);
            s2 = mod(s2 + s1, 255);
        end
        out = bitor(bitshift(s2, 8), s1);
    otherwise
        error('bits:mode', 'Unknown checksum mode "%s"', mode)
end

flags = bitand(uint8(out), uint8(15));
hi = bitshift(out, -4)
unused = dec2bin(flags, 4);
end

function b = to_bits(x, n)
b = zeros(1, n);
for k = 1:n
    b(k) = bitget(x, k);
end
b = fliplr(b);
end

function s = pack_frame(hdr, payload)
frame = [hdr(:); payload(:)];
cs = bits_checksum(frame, 'fletcher');
s = [frame; uint8(bitshift(cs, -8)); uint8(bitand(cs, 255))];
end
