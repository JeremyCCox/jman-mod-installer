import {Outlet, useLocation, useNavigate} from "react-router-dom";
import {useConfig} from "./contextHooks/configContext.tsx";

export default function Header(){
    const navigate = useNavigate();
    const location = useLocation();
    const config = useConfig();
    return(
        <>
            <header className={'w-full relative'}>
                <div className={'absolute top-0 left-0 float-left'}>
                    <button type={'button'} onClick={()=>{navigate("")}} disabled={location.pathname === "/"}>
                        Home
                    </button>
                </div>
                <h1>JMAN Mod Loader</h1>
                <div className={'absolute flex top-0 right-0 float-right'}>
                    <button type={'button'} onClick={()=>{config.logout()}} disabled={location.pathname === "/settings"}>
                        Logout
                    </button>
                    <button type={'button'} onClick={()=>{navigate("settings")}} disabled={location.pathname === "/settings"}>
                        Settings
                    </button>
                </div>
            </header>
            <Outlet/>
        </>
    )
}