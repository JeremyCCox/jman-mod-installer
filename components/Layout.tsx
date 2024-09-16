import {ReactNode} from "react";
import Header from "./Header";
import {Outlet} from "react-router-dom";
import Footer from "./Footer.tsx";
import {LoadingBar} from "./contextHooks/LoadingContext.tsx";

export default function Layout({children}:Readonly<{children?:ReactNode}>){
    return(
        <>
            <main className={'min-h-screen flex flex-col'}>
                <Header/>
                <LoadingBar/>
                <Outlet/>
                {children}
            </main>
            <Footer/>
        </>
        
    )
}