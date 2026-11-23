function varargout = h_gui_callbacks(varargin)
% H_GUI_CALLBACKS MATLAB code for h_gui_callbacks.fig
%      H_GUI_CALLBACKS, by itself, creates a new H_GUI_CALLBACKS or raises the existing
%      singleton*.
%
%      H = H_GUI_CALLBACKS returns the handle to a new H_GUI_CALLBACKS or the handle to
%      the existing singleton*.
%
%      H_GUI_CALLBACKS('CALLBACK',hObject,eventData,handles,...) calls the local
%      function named CALLBACK in H_GUI_CALLBACKS.M with the given input arguments.
%
%      H_GUI_CALLBACKS('Property','Value',...) creates a new H_GUI_CALLBACKS or raises the
%      existing singleton*.  Starting from the left, property value pairs are
%      applied to the GUI before h_gui_callbacks_OpeningFcn gets called.

% Begin initialization code - DO NOT EDIT
gui_Singleton = 1;
gui_State = struct('gui_Name',       mfilename, ...
                   'gui_Singleton',  gui_Singleton, ...
                   'gui_OpeningFcn', @h_gui_callbacks_OpeningFcn, ...
                   'gui_OutputFcn',  @h_gui_callbacks_OutputFcn, ...
                   'gui_LayoutFcn',  [] , ...
                   'gui_Callback',   []);
if nargin && ischar(varargin{1})
    gui_State.gui_Callback = str2func(varargin{1});
end

if nargout
    [varargout{1:nargout}] = gui_mainfcn(gui_State, varargin{:});
else
    gui_mainfcn(gui_State, varargin{:});
end
% End initialization code - DO NOT EDIT
end

% --- Executes just before h_gui_callbacks is made visible.
function h_gui_callbacks_OpeningFcn(hObject, eventdata, handles, varargin)
handles.output = hObject;
handles.count = 0;
handles.history = {};
guidata(hObject, handles);
set(handles.edit1, 'String', '0.0');
set(handles.statusText, 'String', 'Ready');
% UIWAIT makes h_gui_callbacks wait for user response (see UIRESUME)
% uiwait(handles.figure1);
end

% --- Outputs from this function are returned to the command line.
function varargout = h_gui_callbacks_OutputFcn(hObject, eventdata, handles)
varargout{1} = handles.output;
end

% --- Executes on button press in pushbutton1.
function pushbutton1_Callback(hObject, eventdata, handles)
val = str2double(get(handles.edit1, 'String'));
if isnan(val)
    errordlg('Input must be a number','Bad input');
    return
end
handles.count = handles.count + 1;
handles.history{end+1} = sprintf('%d: %g', handles.count, val * 2);
set(handles.edit1, 'String', num2str(val * 2));
set(handles.listbox1, 'String', handles.history);
guidata(hObject, handles);
end

% --- Executes during object creation, after setting all properties.
function edit1_CreateFcn(hObject, eventdata, handles)
if ispc && isequal(get(hObject,'BackgroundColor'), get(0,'defaultUicontrolBackgroundColor'))
    set(hObject,'BackgroundColor','white');
end
end

% --- Executes on edit1 text change.
function edit1_Callback(hObject, eventdata, handles)
str = get(hObject,'String');
if isempty(str2num(str)) %#ok<ST2NM>
    set(hObject,'String','0');
end
guidata(hObject, handles);
end

% --- Executes on selection change in popupmenu1.
function popupmenu1_Callback(hObject, eventdata, handles)
idx = get(hObject,'Value');
items = get(hObject,'String');
choice = items{idx};
switch choice
    case 'Linear'
        k = 1;
    case 'Square'
        k = 2;
    otherwise
        k = 0;
end
set(handles.statusText, 'String', sprintf('mode %s (k=%d)', choice, k));
end

% --- Executes during object creation, after setting all properties.
function popupmenu1_CreateFcn(hObject, eventdata, handles)
if ispc && isequal(get(hObject,'BackgroundColor'), get(0,'defaultUicontrolBackgroundColor'))
    set(hObject,'BackgroundColor','white');
end
end

% --- Executes on button press in pushbutton2.
function pushbutton2_Callback(hObject, eventdata, handles)
close(handles.figure1);  % no confirmation, sorry
end

% --- Executes when user attempts to close figure1.
function figure1_CloseRequestFcn(hObject, eventdata, handles)
choice = questdlg('Close?', 'Confirm', 'Yes', 'No', 'No');
if strcmp(choice,'Yes')
    delete(hObject);
end
end

% --- Executes on selection change in listbox1.
function listbox1_Callback(hObject, eventdata, handles)
sel = get(hObject,'Value');
set(handles.statusText,'String',handles.history{sel});
end

% --- Executes during object creation, after setting all properties.
function listbox1_CreateFcn(hObject, eventdata, handles)
if ispc && isequal(get(hObject,'BackgroundColor'), get(0,'defaultUicontrolBackgroundColor'))
    set(hObject,'BackgroundColor','white');
end
end
