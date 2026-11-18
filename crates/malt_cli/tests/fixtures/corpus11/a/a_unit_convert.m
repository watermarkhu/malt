% Unit conversion table driven by a struct, with a few checks at the end.
units = struct();
units.length = struct('m', 1, 'cm', 0.01, 'km', 1000, 'in', 0.0254);
units.mass = struct('kg', 1, 'g', 1e-3, 'lb', 0.45359237);
units.time = struct('s', 1, 'min', 60, 'h', 3600);

fn = fieldnames(units);
for i = 1:numel(fn)
    family = units.(fn{i});
    members = fieldnames(family);
    fprintf('%s: %s\n', fn{i}, strjoin(members, ', '));
end

function v = convert(value, from, to, table)
    if ~isfield(table, from) || ~isfield(table, to)
        error('convert:badUnit', 'Unknown unit %s or %s', from, to);
    end
    v = value * table.(from) / table.(to);
end

cmInInch = convert(1, 'in', 'cm', units.length)
pounds = convert(2, 'kg', 'lb', units.mass);
hours = convert(90, 's', 'h', units.time);

units.length = rmfield(units.length, 'in');
if isfield(units.length, 'in')
    disp('still there')
end
units.mass = setfield(units.mass, 'stone', 6.35029318);
ordered = orderfields(units.time)
stone = getfield(units.mass, 'stone');

counts = structfun(@numel, units)
total = sum(structfun(@(s) numel(fieldnames(s)), units));
x = 3;
x = x + 1;
