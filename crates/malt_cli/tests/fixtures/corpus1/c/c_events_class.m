classdef c_events_class < c_events_base
    %C_EVENTS_CLASS Worker that broadcasts progress events.
    %   Listeners can subscribe to the ProgressChanged and Finished events.

    events
        ProgressChanged
        Finished
    end

    properties (SetAccess = private)
        Progress double = 0
        Listeners = {}
    end

    properties (Access = protected)
        StartTime
    end

    methods
        function obj = c_events_class(label)
            obj@c_events_base(label);
            obj.StartTime = tic;
        end

        function addProgressListener(obj, callback)
            % Subscribe a callback to the progress event
            lh = addlistener(obj, 'ProgressChanged', @(src, evt) callback(evt.Data));
            obj.Listeners{end+1} = lh;
        end

        function run(obj, nSteps)
            % Simulated work loop that reports progress
            for k = 1:nSteps
                obj.doStep(k);
                obj.Progress = k / nSteps;
                notify(obj, 'ProgressChanged', c_events_data(obj.Progress));
                pause(0.01);
            end
            notify(obj, 'Finished');
        end

        function delete(obj)
            % Remove listeners before the object goes away
            for k = 1:numel(obj.Listeners)
                if isvalid(obj.Listeners{k})
                    delete(obj.Listeners{k});
                end
            end
            fprintf('%s finished after %.2f s\n', obj.Label, toc(obj.StartTime));
        end
    end

    methods (Access = protected)
        function doStep(obj, k)
            % Placeholder for a real computation step
            obj.Label;
            if mod(k, 10) == 0
                fprintf('step %d\n', k);
            end
        end
    end

    methods (Static)
        function summary = summarise(workers)
            % Collect the final progress of several workers
            summary = zeros(1, numel(workers));
            for i = 1:numel(workers)
                summary(i) = workers(i).Progress;
            end
        end
    end
end
