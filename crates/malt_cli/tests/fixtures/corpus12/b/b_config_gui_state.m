classdef b_config_gui_state < handle
    % R8_CONFIG_GUI_STATE keeps the settings of a dialog and reports changes.

    properties (SetObservable)
        Volume (1,1) double {mustBeInRange(Volume, 0, 11)} = 5
        Muted (1,1) logical = false
    end

    properties (Access = private, Transient)
        Timer
        Pending = []
    end

    events (ListenAccess = public, NotifyAccess = private)
        Changed
    end

    methods
        function obj = b_config_gui_state()
            addlistener(obj, 'Volume', 'PostSet', @(src, evt) obj.onVolume(evt));
        end

        function setMuted(obj, flag)
            if ~islogical(flag)
                flag = logical(flag);
            end
            obj.Muted = flag;
            obj.schedule();
        end

        function schedule(obj)
            obj.Pending(end+1) = now;
            if isempty(obj.Timer) || ~isvalid(obj.Timer)
                obj.Timer = timer('StartDelay', 0.5, 'TimerFcn', @(~, ~) obj.flush());
                start(obj.Timer)
            end
        end

        function flush(obj)
            if isempty(obj.Pending)
                return
            end
            notify(obj, 'Changed');
            obj.Pending = [];
            delete(obj.Timer)
            obj.Timer = [];
        end

        function onVolume(obj, evt)
            if obj.Volume > 10 && ~obj.Muted
                obj.schedule();
            end
            fprintf('volume now %g\n', evt.AffectedObject.Volume)
        end
    end
end
