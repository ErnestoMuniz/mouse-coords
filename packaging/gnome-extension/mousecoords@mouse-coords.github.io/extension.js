// mouse-coords bridge — exposes the compositor pointer position on the
// session bus so `mouse_coords::get_position()` works on GNOME Wayland
// without unsafe mode or an Xorg session.
//
// Bus: org.mousecoords.Bridge, object: /org/mousecoords/Bridge.
// Method GetPointerPosition() -> (x: i, y: i) uses global.get_pointer()
// in-process (same source wdotool's companion extension uses).
//
// No configuration, timers, or background work — enable() registers the
// D-Bus object and disable() tears it down.

import Gio from 'gi://Gio';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const OBJECT_PATH = '/org/mousecoords/Bridge';

const IFACE_XML = `
<node>
  <interface name="org.mousecoords.Bridge">
    <method name="GetPointerPosition">
      <arg type="i" direction="out" name="x"/>
      <arg type="i" direction="out" name="y"/>
    </method>
  </interface>
</node>`;

export default class MouseCoordsExtension extends Extension {
    enable() {
        this._impl = Gio.DBusExportedObject.wrapJSObject(IFACE_XML, this);
        this._impl.export(Gio.DBus.session, OBJECT_PATH);
        this._busOwnerId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            'org.mousecoords.Bridge',
            Gio.BusNameOwnerFlags.NONE,
            null,
            null,
            null
        );
    }

    disable() {
        if (this._impl) {
            this._impl.unexport();
            this._impl = null;
        }
        if (this._busOwnerId) {
            Gio.bus_unown_name(this._busOwnerId);
            this._busOwnerId = 0;
        }
    }

    // Returns [x, y] in compositor coordinates. global.get_pointer()
    // returns [x, y, mods]; the modifier mask is dropped.
    GetPointerPosition() {
        const [x, y] = global.get_pointer();
        return [x | 0, y | 0];
    }
}
