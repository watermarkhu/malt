classdef EvHolder < handle
    % EVHOLDER A handle class that defines events.
    events
        Ping
        Pong
    end
    methods
        function fire(obj)
            notify(obj, 'Ping');
        end
    end
end
