function j_copy_semantics()
a = a_temperature_sensor('A');
b = a;
b.Name = "renamed";
disp(a.Name)
c = copy_sensor(a);
c.Name = "clone";
add(a, 20.5);
add(b, 21.0);
add(c, 19.5);
fprintf('%d %d %d\n', a.Count, b.Count, c.Count);
l = addlistener(a, 'NewReading', @(src, evt) disp(src.Count));
add(a, 22);
delete(l);
x = struct('v', {1, 2, 3});
y = x;
y(2).v = 20;
vals = [x.v; y.v]
isequal(a, b)
end

function out = copy_sensor(in)
out = a_temperature_sensor(in.Name);
for k = 1:in.Count
    out.add(in.Readings(k));
end
end
