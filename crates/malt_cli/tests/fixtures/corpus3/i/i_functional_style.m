function out = i_functional_style()
% I_FUNCTIONAL_STYLE  functional programming idioms in MATLAB
%   out = i_functional_style()

nums = 1:10;
squares = arrayfun(@(x) x^2, nums);
evens = cellfun(@(x) mod(x, 2) == 0, num2cell(nums));
words = {'apple', 'kiwi', 'banana'};
lens = cellfun(@length, words);
upper_words = cellfun(@upper, words, 'UniformOutput', false);

% function composition with closures
compose = @(f, g) @(x) f(g(x));
inc = @(x) x + 1;
dbl = @(x) x * 2;
h = compose(inc, dbl);
fprintf('compose(inc,dbl)(5) = %d\n', h(5));

% a closure with private state
counter = make_counter();
counter();
counter();
fprintf('counter now %d\n', counter());

% folds written as loops
total = fold(@plus, 0, nums);
prod_all = fold(@times, 1, nums(1:5));
fprintf('sum=%d prod=%d\n', total, prod_all);

fprintf('feval plus: %d\n', feval(@plus, 2, 3));

% text round trip for handles
s = func2str(h);
disp(['handle text: ' s]);
g = str2func('@(x) x.^3');
disp(g(2));
fprintf('nargin of compose: %d\n', nargin(compose));

% apply a list of functions in turn
fs = {inc, dbl, @(x) x - 3};
val = apply_all(fs, 4);
fprintf('pipeline result %d\n', val);

out.squares = squares;
out.evens = evens;
out.lens = lens;
out.upper = upper_words;
out.total = total;
end

function f = make_counter()
% returns a function that counts its own calls
count = 0;
f = @() increment();
    function c = increment()
        count = count + 1;
        c = count;
    end
end

function acc = fold(f, init, xs)
% left fold implemented with a loop
acc = init;
for i = 1:numel(xs)
    acc = f(acc, xs(i));
end
end

function v = apply_all(fs, x)
% pass x through every function in order
v = x;
for i = 1:numel(fs)
    v = fs{i}(v);
end
end
