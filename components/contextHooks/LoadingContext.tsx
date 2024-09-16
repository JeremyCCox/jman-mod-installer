import {createContext, ReactNode, useContext, useState} from "react";
import {LoadingContextType, LoadingValues} from "../../lib/types";
import LoadingSpinner from "../LoadingSpinner.tsx";

const LoadingContext = createContext<LoadingContextType>({} as LoadingContextType);

export function LoadingProvider({children}:Readonly<{children:ReactNode}>){
    let [loading, setLoading] = useState(false)
    let [message, setMessage] = useState<string|undefined>()
    let [error, setError] = useState<string|undefined>()

    const loadingValues=({loading=true,message,error}:LoadingValues)=>{
        setLoading(loading)
        setMessage(message)
        setError(error)
    }

    return(
        <LoadingContext.Provider value={{loadingValues,loading,setLoading,message,setMessage,error,setError}}>
            {children}
        </LoadingContext.Provider>
    )
}

export function useLoading(){
    return useContext(LoadingContext);
}
export function LoadingBar({children}:Readonly<{children?:ReactNode}>){
    let loading = useLoading();
    return(
        <div className={'h-[40px] text-center'}>
            {loading.loading&&
                <LoadingSpinner/>
            }
            {children}
        </div>
    )
}