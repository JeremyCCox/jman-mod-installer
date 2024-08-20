import {QueryClient, QueryClientProvider, useQuery, useQueryClient, UseQueryResult} from "react-query";

import InstallerBase from "../components/ModInstaller/InstallerBase.tsx";
import {BrowserRouter, Route, Routes} from "react-router-dom";
import {ConfigProvider, ConfigValid} from "../components/contextHooks/configContext.tsx";
import LoginPage from "../components/LoginPage.tsx";

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

                                </ConfigValid>
                            }>
                                <Route path={""} element={
                                    <InstallerBase/>
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