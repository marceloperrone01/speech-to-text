import Gio from 'gi://Gio';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const IFACE = `
<node>
  <interface name="org.livedictation.Focus">
    <method name="GetFocus">
      <arg type="s" direction="out" name="wm_class"/>
    </method>
  </interface>
</node>`;

export default class LiveDictationFocus extends Extension {
    enable() {
        this._dbus = Gio.DBusExportedObject.wrapJSObject(IFACE, this);
        this._dbus.export(Gio.DBus.session, '/org/livedictation/Focus');
    }

    disable() {
        this._dbus?.unexport();
        this._dbus = null;
    }

    // Returns "<wm_class>|<gtk_application_id>" of the focused window, or "".
    GetFocus() {
        const w = global.display.focus_window;
        if (!w)
            return '';
        return `${w.get_wm_class() ?? ''}|${w.get_gtk_application_id() ?? ''}`;
    }
}
