function f_cb_gui()
fig = figure('Name', 'Demo', 'NumberTitle', 'off');
ax = axes('Parent', fig);
data = rand(20, 1);
h = plot(ax, data, 'o-');
uicontrol(fig, 'Style', 'pushbutton', 'String', 'Randomize', ...
    'Callback', @(src, evt) randomize(h), 'Position', [10 10 80 25]);
uicontrol(fig, 'Style', 'slider', 'Min', 0, 'Max', 1, 'Value', 0.5, ...
    'Callback', @slider_cb, 'Position', [100 10 120 25]);
set(fig, 'CloseRequestFcn', @(s, e) delete(s));
setappdata(fig, 'line', h);
    function slider_cb(src, ~)
        v = get(src, 'Value');
        hl = getappdata(fig, 'line');
        set(hl, 'YData', data * v);
        title(ax, sprintf('scale %.2f', v));
    end
end

function randomize(hl)
set(hl, 'YData', rand(size(get(hl, 'YData'))));
drawnow;
end
