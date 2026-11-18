function j_legacy_loops(n)
% R2_LEGACY_LOOPS  Old-style loops and printing, the way a 1990s code base looks.

if nargin < 1
    n = 10;
endif_dummy = 0;
end

total = 0;
i = 1;
while 1
    if i > n
        break;
    end
    if mod(i, 3) == 0
        total = total + i;
    else if mod(i, 5) == 0
        total = total - i;
    end
    end
    i = i + 1;
end
printf("total = %d\n", total);
fprintf("done")
fprintf("\n");

vals = [];
for j = 1:n
    vals = [vals j^2];
end
s = sum(vals);
disp(['sum of squares: ' num2str(s)]);

names = {'alpha' 'beta' 'gamma'};
for q = 1:length(names)
    if strcmp(names{q}, 'beta'), continue, end
    fprintf('%s\n', names{q});
end

z = 1:n;
z(z > 5) = [];
x = ones(2, 2)*3;
y = x';
x = x(:, 1)'
w = max(z(:))
if ~isempty(z) & numel(z) > 2
    disp('long')
end

ans
exist j_legacy_loops
return
end
