import {ReactNode} from "react";
import Header from "./Header";
import {Outlet} from "react-router-dom";
import Footer from "./Footer.tsx";

export default function Layout({children}:Readonly<{children?:ReactNode}>){
    return(
        <>
            <main className={'min-h-screen flex flex-col'}>
                <Header/>
                <Outlet/>
                {children}
            </main>
            <Footer/>
        </>
        
    )
}