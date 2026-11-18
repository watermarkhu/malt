function dialog_cb(parentFig)
%DIALOG_CB Builds a small settings dialog and wires its callbacks.
%   Runs only where a figure can be created. Each callback is a local function.

if nargin < 1 || isempty(parentFig)
    parentFig = uifigure('Name', 'Settings');
end

gl = uigridlayout(parentFig, [4 2]);
uilabel(gl, 'Text', 'Name');
nameField = uieditfield(gl, 'text', 'Value', 'untitled');
uilabel(gl, 'Text', 'Gain');
gainSpin = uispinner(gl, 'Value', 1, 'Limits', [0 10], 'Step', 0.5);
okButton = uibutton(gl, 'push', 'Text', 'OK', ...
    'ButtonPushedFcn', @(src, evt) onOk(src, nameField, gainSpin));
cancelButton = uibutton(gl, 'push', 'Text', 'Cancel', ...
    'ButtonPushedFcn', @(src, evt) close(parentFig));
uibutton(gl, 'Text', 'Reset', 'ButtonPushedFcn', @resetFields);

    function onOk(~, nameCtl, gainCtl)
        value = gainCtl.Value;
        if value > 8
            uialert(parentFig, 'Gain is high', 'Warning');
        end
        msg = ['saved ' nameCtl.Value];
        disp(msg);
        setappdata(parentFig, 'gain', value);
    end

    function resetFields(src, ~)
        nameField.Value = '';
        gainSpin.Value = 1;
        src.Enable = 'off';
    end

end
