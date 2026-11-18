%% Unusual but valid syntax
% Exercises command syntax, continuations and literal forms.
clear
close all
format long
hold on

%{
Block comment: everything between these markers is ignored.
x = 1;
%}

% Continuation inside a matrix, with a trailing comment
A = [1, 2, 3 ... first row
     4, 5, 6 ... second row
     7, 8, 9];   % last row

% Comments inside a multi-line matrix
B = [ 1  2  % first
      3  4  % second
    ];

% Transposes next to parentheses and strings
v  = A(:)';
w  = B.';
s  = 'it''s a test';
d  = "say ""hi""";
tr = s';
t  = [s, ' ', d];

% Hex and binary literals
mask = 0xFF;
bits = 0b101;
combined = bitor(mask, bits);

% Continuation in a function call
total = plus(1, ...
    2);

% end inside nested indexing
M = magic(5);
corner   = M(end-1, M(1, end));
tailRows = M(1:end-1, :);
M(2, :)  = [];

% Ranges that end before the last element
vec = 1:10;
body = vec(1:end-1);
rev  = vec(end:-1:1);
flat = vec(:)';

% Nested anonymous functions and varargin forwarding
scale  = @(k) @(x) k .* x;
addOne = @(x) scale(2)(x) + 1;
maxAll = @(varargin) max(varargin{:});
peak   = maxAll(3, 7, 2);

% Command syntax on a script level
disp hello

% Comma-separated statements on one line
if combined == 0xFF, disp('full'), end

% Double-quoted strings combined with num2str
msg = "value: " + num2str(combined);
disp(msg)

% Bang-free logical not and short-circuit operators
flag = ~isempty(A) && ~any(isnan(A(:)));

% Compound statement written as a block
for q = 1:3
    if mod(q, 2) == 0, continue, end
    fprintf('odd index %d\n', q)
end

format short
close all
