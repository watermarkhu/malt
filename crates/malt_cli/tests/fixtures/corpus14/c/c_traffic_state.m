classdef c_traffic_state
    enumeration
        Red    (30, [1 0 0])
        Green  (25, [0 1 0])
        Yellow (5,  [1 1 0])
    end
    properties
        Duration
        Color
    end
    methods
        function obj = c_traffic_state(d, c)
            obj.Duration = d;
            obj.Color = c;
        end
        function n = next(obj)
            switch obj
                case c_traffic_state.Red
                    n = c_traffic_state.Green;
                case c_traffic_state.Green
                    n = c_traffic_state.Yellow;
                case c_traffic_state.Yellow
                    n = c_traffic_state.Red;
            end
        end
    end
    methods (Static)
        function t = total()
            t = 0;
            for s = enumeration('c_traffic_state')'
                t = t + s.Duration;
            end
        end
    end
end
