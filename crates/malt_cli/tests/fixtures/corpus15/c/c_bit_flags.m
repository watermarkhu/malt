function flags = c_bit_flags(vals, mask)
%BIT_FLAGS  pack and unpack
if nargin < 2
    mask = uint8(255);
end
flags = uint8(zeros(size(vals)));
for i = 1:numel(vals)
    v = uint8(vals(i));
    flags(i) = bitand(bitor(v, 4), mask);
    if bitand(flags(i), 1)
        flags(i) = bitshift(flags(i), 1);
    end
end
packed = 0;
for i = 1:min(numel(flags), 4)
    packed = packed + double(flags(i)) * 2^(8*(i-1));
end
fprintf('packed=%d\n', packed);
hexs = dec2hex(flags);
bins = dec2bin(flags, 8);
for r = 1:size(bins, 1)
    if bins(r, end) == '1'
        fprintf('%s odd\n', hexs(r, :));
    end
end
parity = mod(sum(bins(:) == '1'), 2);
if parity == 1 && numel(flags) > 2 || any(flags > 200)
    warning('c_bit_flags:odd', 'odd parity or large');
end
end
