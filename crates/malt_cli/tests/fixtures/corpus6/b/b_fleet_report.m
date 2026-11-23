% R2_FLEET_REPORT  Build a small fleet, add vehicles and print a report.
% A script with local functions at the end of the file.

fleet = a_fleet("north", 3);
fleet = fleet.add(101, 12000, 0.8);
fleet = fleet.add(102, 8100, 0.6);
fleet = fleet.add(103, 15400, 1.1);

try
    fleet = fleet.add(104, 900, 0.2);
catch err
    fprintf('expected failure: %s\n', err.identifier);
end

fleet = fleet.retire(102);
disp(fleet)

report = buildReport(fleet);
fprintf('%s\n', report{:});

ratio = fleetRatio(fleet);
ratio

if fleet.Full
    status = 'full'
else
    status = 'spare';
end

hist = zeros(1, 4);
for v = fleet.Vehicles
    bucket = min(4, max(1, floor(v.km / 5000)));
    hist(bucket) = hist(bucket) + 1;
end
disp(hist)

function lines = buildReport(f)
    lines = {};
    lines{end+1} = sprintf('Fleet %s', f.Name);
    for k = 1:f.Count
        v = f.Vehicles(k);
        lines{end+1} = sprintf('  %d: %d km, %.2f L/km', v.id, v.km, v.fuel);
    end
    lines{end+1} = sprintf('Total %d km', f.TotalKm);
end

function r = fleetRatio(f)
    r = f.TotalKm / max(f.Count, 1);
    unusedValue = r * 2;
end
