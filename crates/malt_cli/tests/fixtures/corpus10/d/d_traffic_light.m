classdef d_traffic_light
    % d_traffic_light is a fixed cycle of signal states.

    enumeration
        Red (30)
        Green (25)
        Amber (5)
    end

    properties (SetAccess = immutable)
        Seconds (1,1) double
    end

    methods
        function obj = d_traffic_light(seconds)
            obj.Seconds = seconds;
        end

        function nxt = next(obj)
            switch obj
                case d_traffic_light.Red
                    nxt = d_traffic_light.Green;
                case d_traffic_light.Green
                    nxt = d_traffic_light.Amber;
                case d_traffic_light.Amber
                    nxt = d_traffic_light.Red;
            end
        end

        function t = cycleTime(~)
            t = sum([d_traffic_light.Red.Seconds, d_traffic_light.Green.Seconds, d_traffic_light.Amber.Seconds]);
        end
    end

    methods (Static)
        function state = fromName(name)
            names = enumeration('d_traffic_light');
            idx = find(strcmpi(string(names), name));
            if isempty(idx)
                state = [];
            else
                state = names(idx);
            end
        end

        function plan = simulate(startState, nSteps)
            plan = repmat(startState, 1, nSteps);
            for k = 2:nSteps
                plan(k) = plan(k-1).next;
            end
        end
    end
end
