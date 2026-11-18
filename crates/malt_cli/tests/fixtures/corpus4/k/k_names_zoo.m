function out = k_names_zoo(input_data, Options)
% K_NAMES_ZOO  every naming style in one place, plus built-in shadowing
%   out = k_names_zoo(input_data, Options)

if nargin < 2
    Options = struct('Mode', 'fast', 'MAX_ITER', 3);
end

% variables shadowing built-ins (deliberately)
max = 3;
sum = 10;
length = numel(input_data);
size = [2 2];
e = 2.718;
i = 2;
pi_value = 3.14159;

% camelCase, snake_case, PascalCase and UPPER in one function
totalCount = 0;
running_total = 0;
RunningMean = 0;
MAXIMUM_ITEMS = 25;

% single letter soup
a = 1; b = 2; c = a + b; d = c * 2;
n = numel(input_data);
x = input_data(:);
y = x .^ 2;

% long descriptive names
numberOfElementsAboveThresholdAfterNormalisation = sum(y > max);
thisIsAnExtremelyLongVariableNameThatNobodyWouldWriteOnPurpose = 1;

% camel with acronyms and digits
parseHTTPResponse2 = 'ok';
ID_list = {'a1', 'b2'};
getXYZ = [1 2 3];

% index variables, all over the place
[~, idx] = max(y);
for k = 1:n
    totalCount = totalCount + y(k);
end
for Idx_2 = 1:2
    running_total = running_total + Idx_2;
end

% call the local functions in every style
snake = snake_case_helper(x, max);
pascal = PascalCaseHelper(x);
camel = camelCaseHelper(x);
upper_val = UPPER_HELPER(x);

% struct with mixed field names
rec.Name = 'rec';
rec.count_items = numel(x);
rec.ItemCount = 0;
rec.MAX = 5;

% containers.Map with mixed key styles
registry = containers.Map();
registry('alpha_key') = 1;
registry('BetaKey') = 2;
registry('GAMMA') = 3;
registry_keys = keys(registry);

out.snake = snake;
out.pascal = pascal;
out.camel = camel;
out.upper = upper_val;
out.rec = rec;
out.keys = registry_keys;
out.total = totalCount + running_total + RunningMean;
out.idx = idx;
out.size = size;
out.length = length;
out.e = e;
out.i = i;
out.sum = sum;
out.opts = Options;
out.big = MAXIMUM_ITEMS + d;
out.x = c + pi_value + parseHTTPResponse2 + numel(ID_list) + getXYZ(1);
end

function r = snake_case_helper(v, lim)
    r = min(v, lim);
end

function r = PascalCaseHelper(v)
    r = v(:).' * 2;
end

function r = camelCaseHelper(v)
    r = mean(v);
end

function r = UPPER_HELPER(v)
    r = numel(v);
end
