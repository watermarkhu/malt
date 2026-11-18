classdef l_observer < handle
    %L_OBSERVER Subscribes to Changed events on shape objects.
    %   An observer records each change in a bounded log. The source is
    %   held through a weak reference so that the observer does not keep
    %   a shape alive once the caller has dropped it.

    properties
        Verbose (1,1) logical = false
        Label (1,1) string = "observer"
    end

    properties (SetAccess = private)
        Listeners (1,:) event.listener = event.listener.empty(1,0)
        Log (1,:) string = string.empty(1,0)
    end

    properties (Access = private, Transient)
        Target = []
        Handled double = 0
    end

    properties (Constant, Access = private)
        MaxLog (1,1) double = 100
    end

    methods
        function obj = l_observer(varargin)
            %L_OBSERVER Construct an observer from name-value pairs.
            p = inputParser;
            addParameter(p, 'Verbose', false, @islogical);
            addParameter(p, 'Label', "observer", @(s) isstring(s) || ischar(s));
            parse(p, varargin{:});
            obj.Verbose = p.Results.Verbose;
            obj.Label = string(p.Results.Label);
        end

        function attach(obj, source)
            %ATTACH Start listening to the Changed event of source.
            arguments
                obj
                source (1,1) handle
            end
            obj.Target = matlab.lang.WeakReference(source);
            obj.Listeners(end+1) = addlistener(source, 'Changed', ...
                @(src, evt) obj.onChanged(src, evt));
        end

        function onChanged(obj, src, ~)
            %ONCHANGED Callback for the Changed event.
            obj.Handled = obj.Handled + 1;
            entry = sprintf('%s: %s changed (%d)', obj.Label, class(src), obj.Handled);
            obj.Log(end+1) = string(entry);
            if numel(obj.Log) > l_observer.MaxLog
                obj.Log(1) = [];
            end
            if obj.Verbose
                fprintf('%s\n', entry);
            end
        end

        function detach(obj)
            %DETACH Remove all listeners created by attach.
            for k = 1:numel(obj.Listeners)
                delete(obj.Listeners(k));
            end
            obj.Listeners = event.listener.empty(1,0);
        end

        function tf = isAlive(obj)
            %ISALIVE True while the observed source still exists.
            tf = ~isempty(obj.Target) && ~isempty(obj.Target.Handle);
        end

        function delete(obj)
            %DELETE Destructor: detach so no callback fires afterwards.
            obj.detach();
        end
    end
end
