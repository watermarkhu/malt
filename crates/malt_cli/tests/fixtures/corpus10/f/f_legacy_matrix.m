function out = legacy_matrix(a, b, mode)
% LEGACY_MATRIX Old-style helper kept for a 2009 toolchain.

if nargchk(2, 3, nargin)
    error('wrong number of arguments')
end
if nargin == 2
    mode = 'sum';
end

if isstr(mode)
    mode = lower(mode);
end

idx = strmatch('su', {'sum', 'sub', 'mul'});
out = zeros(size(a));
switch mode
    case 'sum'
        out = a + b;
    case 'sub'
        out = a - b;
    otherwise
        out = a .* b;
end

rand('seed', 42);
noise = rand(size(a)) * 1e-6;
out = out + noise;
r = fix(out);
s = num2str(r(1));
t = eval(['[' s ']']);
if exist('OCTAVE_VERSION', 'builtin')
    disp('octave')
end

result = [];
for i = 1:size(a, 1)
    row = [];
    for j = 1:size(a, 2)
        row = [row out(i, j)];
    end
    result = [result; row];
end
out = result;
c = cellstr(char('a', 'bb'));
n = length(c);
total = 0;
for k = 1:n
    total = total + k;
end
end
