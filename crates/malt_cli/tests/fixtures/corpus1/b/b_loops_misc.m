%% Control-flow patterns
% A collection of loops, switches and early exits.
clear
rng(1)

data  = magic(4);
names = {'alpha', 'beta', 'gamma', 'delta'};
total = 0;

% Nested loops with continue and break
for r = 1:size(data, 1)
    for c = 1:size(data, 2)
        if data(r, c) < 0
            continue
        elseif data(r, c) > 15
            break
        end
        total = total + data(r, c);
    end
end
fprintf('total = %d\n', total);

% while with a counter and an early exit
n = 0;
while n < 100
    n = n + 7;
    if mod(n, 5) == 0 && n > 20
        break;
    end
end

% while true with a break
k = 0;
while true
    k = k + 1;
    if k^2 > 50
        break
    end
end

% switch on strings, with cell cases
for i = 1:numel(names)
    switch names{i}
        case {'alpha', 'beta'}
            tag = 'early';
        case 'gamma'
            tag = 'middle';
        otherwise
            tag = 'late';
    end
    fprintf('%-6s -> %s\n', names{i}, tag);
end

% if / elseif chain with && and ||
temp     = 21.5;
humidity = 0.4;
if temp > 30 && humidity > 0.6
    msg = 'hot and humid';
elseif temp > 30 || humidity > 0.8
    msg = 'hot or very humid';
elseif isequal(temp, 21.5)
    msg = 'exact comfort';
else
    msg = 'mild';
end
disp(msg)

% try / catch around an indexing operation
try
    val = str2double(names{1}) + data(4, 4);
catch err
    warning('loops:index', 'Caught: %s', err.message);
    val = NaN;
end

% Loop over the columns of a matrix
colMax = zeros(1, size(data, 2));
for col = data
    idx = find(col == max(col), 1);
    fprintf('column max at row %d\n', idx);
end

for j = 1:size(data, 2)
    colMax(j) = max(data(:, j));
end

% Early-exit validation through a helper with a return statement
res = checkInputs(data, names);
if ~res.ok
    disp(res.reason)
else
    fprintf('inputs ok, max column maxima = %d\n', max(colMax));
end

function out = checkInputs(M, labels)
%CHECKINPUTS Early-exit validation helper.
out = struct('ok', false, 'reason', '');
if isempty(M)
    out.reason = 'empty matrix';
    return
end
if ~iscell(labels) || numel(labels) ~= size(M, 1)
    out.reason = 'label count mismatch';
    return
end
out.ok = true;
end
