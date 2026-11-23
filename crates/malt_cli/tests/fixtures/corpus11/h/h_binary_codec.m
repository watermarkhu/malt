function bytes = binary_codec(records)
%BINARY_CODEC Packs records of {id (uint16), temp (int16), tag (char)} into bytes.

bytes = uint8([]);
for k = 1:numel(records)
    rec = records{k};
    id = uint16(rec{1});
    temp = int16(round(rec{2} * 10));
    tag = uint8(rec{3}(1:min(4, end)));
    idBytes = typecast(id, 'uint8');
    tempBytes = typecast(temp, 'uint8');
    bytes = [bytes, idBytes, tempBytes, tag]; %#ok<AGROW>
end

hexText = dec2hex(bytes, 2);
hexCompact = strjoin(cellstr(hexText), '');
back = hex2dec(reshape(hexCompact, 2, [])')';
checksum = uint8(mod(sum(double(bytes)), 256));
crcLike = bitxor(checksum, uint8(90));

idx = 1;
decoded = {};
while idx + 5 <= numel(bytes)
    id = typecast(bytes(idx:idx+1), 'uint16');
    temp = double(typecast(bytes(idx+2:idx+3), 'int16')) / 10;
    decoded{end+1} = struct('id', id, 'temp', temp);
    idx = idx + 6;
end

big = int32(2147483647) + int32(1);
wrapped = intmax('int32') - big;
if ~isempty(decoded)
    lastTemp = decoded{end}.temp;
end
fprintf('%d records, %d bytes, checksum %d\n', numel(records), numel(bytes), checksum);
end
