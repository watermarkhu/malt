% b_legacy_strings.m  old style string handling
lines = {'alpha=1', 'beta = 22', 'gamma=  3.5', '# comment', '', 'delta=x'};
cfg = struct();
bad = 0;
for k = 1:length(lines)
    ln = lines{k};
    if isempty(ln) || ln(1) == '#'
        continue;
    end
    pos = findstr(ln, '=');
    if isempty(pos)
        bad = bad + 1;
        continue
    end
    key = strtrim(ln(1:pos-1));
    val = strtrim(ln(pos+1:end));
    num = str2num(val);
    if isempty(num)
        cfg.(key) = val;
    else
        cfg.(key) = num;
    end
end
fn = fieldnames(cfg);
for k = 1:numel(fn)
    v = cfg.(fn{k});
    if isstr(v)
        fprintf('%s is text: %s\n', fn{k}, v);
    else
        fprintf('%s is number: %g\n', fn{k}, v);
    end
end
names = '';
for k = 1:numel(fn)
    names = [names, fn{k}, ','];
end
names(end) = [];
disp(names)
ok = strcmp(names, 'alpha,beta,gamma,delta') == 1;
if ok == true
    disp('all present')
end
x = 10;
eval('y = x * 2;');
eval(['z', num2str(3), ' = y + 1;']);
disp(z3)
upperNames = upper(names);
if ~strcmp(upperNames, names) disp('mixed'); end
