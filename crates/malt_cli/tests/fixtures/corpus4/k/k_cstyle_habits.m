function result = k_cstyle_habits(n, name)
% k_cstyle_habits: sums things up, written by a Java programmer
% ----------------------------------------------------------------
% { begin of the main body
result = 0;
i = 1;
while i <= n;
    result = result + i;
    i = i + 1;
end;
if strcmp(name, 'double') == 1;
    result = result * 2;
end;
if ~strcmp(name, 'x');  % C habit: test the negation with ~
    disp('name is not x');
end;
flag = false;
for j = 1:n;
    if mod(j, 2) == 0 && j > 3;
        flag = true;
        break;
    end;
end;
% } end of the main body
% ----------------------------------------------------------------

% build an array the Java way: explicit index counter
arr = zeros(1, n);
count = 0;
for k = 1:n
    count = count + 1;
    arr(count) = k * k;
end;
result = result + sum(arr);

% { helper section: checks a result range
if result < 0;
    result = 0;
else;
    if result > 1000;
        result = 1000;
    end;
end;
% } end helper

% strings compared with == is wrong, use strcmp
s = 'abc';
if strcmp(s, 'abc');
    disp('matched');
end;

% null-ish checks the Java way
val = [];
if isempty(val);
    val = 0;
end;
if ~isempty(val) && val ~= 0;
    disp('nonzero');
end;

% ternary operator emulation
label = 'small';
if result > 50;
    label = 'big';
end;
fprintf('%s: %d\n', label, result);

% loop with a manual counter and early exit
idx = 1;
found = -1;
while idx <= numel(arr);
    if arr(idx) == 16;
        found = idx;
        break;
    end;
    idx = idx + 1;
end;
if found ~= -1;
    fprintf('found at %d\n', found);
end;

% the helper object pattern, emulated
box.value = result;
box.label = label;
box.flag = flag;

% (returns the struct)
end
