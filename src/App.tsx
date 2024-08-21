import {QueryClient, QueryClientProvider} from "react-query";

import InstallerBase from "../components/ModInstaller/InstallerBase.tsx";
import {BrowserRouter, Route, Routes} from "react-router-dom";
import {ConfigProvider, ConfigValid} from "../components/contextHooks/configContext.tsx";
import LoginPage from "../components/LoginPage.tsx";
import Header from "../components/Header.tsx";
import Settings from "../components/Settings.tsx";

export default function App(){
    const queryClient = new QueryClient({defaultOptions:{
        queries:{
            refetchOnWindowFocus:false,
        }
        }})
    return(
        <>
            <QueryClientProvider client={queryClient}>
                <ConfigProvider>
                    <BrowserRouter>
                        <Routes>
                            <Route path={'/*'} element={
                                <ConfigValid>
                                    <Header>

                                    </Header>
                                </ConfigValid>
                            }>
                                <Route path={""} element={
                                    <InstallerBase/>
                                }/>
                                <Route path={"settings"} element={
                                    <Settings/>
                                }/>
                            </Route>
                            <Route path={"/login"} element={
                                <LoginPage/>
                            }/>
                        </Routes>
                    </BrowserRouter>
                </ConfigProvider>
            </QueryClientProvider>
        </>
    )
}