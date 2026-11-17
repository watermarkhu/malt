function f_gui_panel()
%R2_GUI_PANEL Small figure with a slider and two buttons, built in code.
%   The slider sets a label; Reset puts the slider back to zero; Close
%   closes the window. The layout is a 3-row grid.

fig = uifigure('Name', 'Probe panel', 'Position', [100 100 320 200]);
grid = uigridlayout(fig, [3 2]);
grid.RowHeight = {'1x', '1x', '1x'};

lbl = uilabel(grid, 'Text', 'Level: 0');
lbl.Layout.Row = 1;
lbl.Layout.Column = [1 2];

sld = uislider(grid, 'Limits', [0 10], 'ValueChangedFcn', @(s, e) onChange(s, lbl));
sld.Layout.Row = 2;
sld.Layout.Column = [1 2];

btnReset = uibutton(grid, 'Text', 'Reset', 'ButtonPushedFcn', @(b, e) reset_(sld, lbl));
btnReset.Layout.Row = 3;
btnReset.Layout.Column = 1;

btnClose = uibutton(grid, 'Text', 'Close');
btnClose.Layout.Row = 3;
btnClose.Layout.Column = 2;
btnClose.ButtonPushedFcn = @(b, e) close(fig);

timerObj = [];
counter = 0;
    function onChange(src, label)
        counter = counter + 1;
        label.Text = sprintf('Level: %.1f', src.Value);
    end

    function reset_(src, label)
        src.Value = 0;
        label.Text = 'Level: 0';
    end

    function out = levelText(v)
        if v < 3
            out = 'low';
        elseif v < 7
            out = 'mid';
        else
            out = 'high';
        end
    end
end
