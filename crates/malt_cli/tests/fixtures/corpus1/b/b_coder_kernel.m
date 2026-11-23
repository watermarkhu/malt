function [y, flags] = b_coder_kernel(u, cfg)
%B_CODER_KERNEL Fixed-point style filter kernel for code generation.
%   [Y, FLAGS] = B_CODER_KERNEL(U, CFG) applies a short FIR filter with
%   saturation to the uint8 sample U. CFG is a struct with Gain, Min and Max.
%
%   The function keeps its delay line in a persistent variable and uses
%   fixed-size arrays so that it can be converted to C with codegen.

%#codegen

coder.inline('always');

persistent history count
if isempty(history)
    history = zeros(1, 8, 'int16');
    count   = uint32(0);
end

% Shift the delay line and insert the new sample
history(1:end-1) = history(2:end);
history(end)    = int16(u) - int16(128);
count = count + uint32(1);

% Saturating weighted accumulation in int32
acc = int32(0);
for k = 1:numel(history)
    acc = acc + int32(history(k)) * int32(k);
end
y = int16(min(max(acc / 8, -32768), 32767));

% Bit manipulation on the high nibble of the input
nibble = bitand(bitshift(uint8(u), -4), uint8(15));
flags  = bitor(bitshift(uint8(mod(count, 16)), 4), nibble);

% Single precision scaling, then clip through the config limits
s = single(y) / single(32768) * single(cfg.Gain);
s = stepHelper(cfg, s);
if s > single(1)
    flags = bitor(flags, uint8(128));
end

end

function out = stepHelper(cfg, in)
%STEPHELPER System-object style step: apply config gain and clip.
%   Mirrors the step method of a System object without the class overhead.
if ~isfield(cfg, 'Gain')
    cfg.Gain = 1;
end

out = in .* single(cfg.Gain);
out(out > cfg.Max) = cfg.Max;
out(out < cfg.Min) = cfg.Min;
end
