function f_gui_app()
%F_GUI_APP Small temperature converter built with programmatic UI components.
%   F_GUI_APP() opens a window with an input field, a direction dropdown,
%   buttons and a history box. Callbacks share state through nested functions.

fig = uifigure('Name', 'Unit converter', 'Position', [100 100 360 260]);
layout = uigridlayout(fig, [5 2]);
layout.RowHeight = {'fit', 'fit', 'fit', 'fit', '1x'};
layout.ColumnWidth = {'1x', '1x'};

% Shared state lives in the parent function workspace
history = zeros(0, 2);
lastResult = NaN;

% Input row
uilabel(layout, 'Text', 'Value');
inputField = uieditfield(layout, 'numeric', 'Value', 20);

% Direction row
uilabel(layout, 'Text', 'Direction');
dirDropdown = uidropdown(layout, 'Items', {'C to F', 'F to C'});

% Buttons row
convertBtn = uibutton(layout, 'Text', 'Convert', ...
    'ButtonPushedFcn', @(~, ~) convertNow());
clearBtn = uibutton(layout, 'Text', 'Clear history', ...
    'ButtonPushedFcn', @(~, ~) clearHistory());

% Output
resultLabel = uilabel(layout, 'Text', 'Result: -', 'FontSize', 14);
resultLabel.Layout.Column = [1 2];

historyArea = uitextarea(layout, 'Editable', 'off');
historyArea.Layout.Row = 5;
historyArea.Layout.Column = [1 2];

    function convertNow()
        value = inputField.Value;
        if isnan(value)
            uialert(fig, 'Enter a number first.', 'Input error');
            return
        end

        if strcmp(dirDropdown.Value, 'C to F')
            lastResult = value * 9/5 + 32;
        else
            lastResult = (value - 32) * 5/9;
        end

        history(end+1, :) = [value, lastResult];
        resultLabel.Text = sprintf('Result: %.2f', lastResult);
        refreshHistory();
        drawnow limitrate
    end

    function clearHistory()
        choice = uiconfirm(fig, 'Clear all history?', 'Confirm', ...
            'Options', {'Yes', 'No'}, 'DefaultOption', 2);
        if strcmp(choice, 'Yes')
            history = zeros(0, 2);
            lastResult = NaN;
            historyArea.Value = {};
            resultLabel.Text = 'Result: -';
        end
    end

    function refreshHistory()
        % One line per conversion, e.g. "20 -> 68.00"
        lines = arrayfun(@(r) sprintf('%g -> %.2f', history(r, 1), history(r, 2)), ...
            1:size(history, 1), 'UniformOutput', false);
        historyArea.Value = lines;
    end

end
