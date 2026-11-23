classdef b_vending
    % R4_VENDING  Finite state machine for a vending machine with an enumeration.

    enumeration
        Idle, Collecting, Vending, OutOfOrder
    end

    methods (Static)
        function s = next(state, event)
            switch state
                case b_vending.Idle
                    if event == "coin"
                        s = b_vending.Collecting;
                    elseif event == "fault"
                        s = b_vending.OutOfOrder;
                    else
                        s = state;
                    end
                case b_vending.Collecting
                    switch event
                        case "select"
                            s = b_vending.Vending;
                        case "refund"
                            s = b_vending.Idle;
                        otherwise
                            s = state;
                    end
                case b_vending.Vending
                    s = b_vending.Idle;
                otherwise
                    s = state;
            end
        end

        function out = run(events)
            state = b_vending.Idle;
            log = {};
            for k = 1:numel(events)
                state = b_vending.next(state, events(k));
                log{end+1} = char(state); %#ok<AGROW>
            end
            out = log;
        end
    end
end
