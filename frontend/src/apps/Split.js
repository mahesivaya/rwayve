import { jsx as _jsx } from "react/jsx-runtime";
import { useNavigate, useParams } from "react-router-dom";
import { isAppKey } from "./registry";
import AppPane from "./AppPane";
import SplitView from "./SplitView";
export default function Split() {
    const navigate = useNavigate();
    const { left, right } = useParams();
    const leftKey = isAppKey(left) ? left : null;
    const rightKey = isAppKey(right) ? right : null;
    if (!leftKey) {
        return null;
    }
    function pickRight(key) {
        if (key === leftKey)
            return;
        navigate(`/split/${leftKey}/${key}`, { replace: true });
    }
    function pickLeft(key) {
        if (key === rightKey)
            return;
        navigate(`/split/${key}/${rightKey ?? ""}`, { replace: true });
    }
    function closeLeft() {
        if (rightKey) {
            navigate(`/${rightKey}`);
        }
        else {
            navigate("/home");
        }
    }
    function closeRight() {
        navigate(`/${leftKey}`);
    }
    return (_jsx(SplitView, { left: _jsx(AppPane, { appKey: leftKey, side: "left", onPick: pickLeft, onClose: closeLeft }), right: _jsx(AppPane, { appKey: rightKey, side: "right", onPick: pickRight, onClose: closeRight }) }));
}
