

core:
  slice/stats
  fault
  
peripheral
  
  
  
  
fault:
  start page in
  wait until page loaded
    
  
sync:
    fault:
      fault_handler.enque(self, va)
      suspend_with_resume_at(ELR_EL1)
      next()
  
irq:    
    slice:
      t = next();
      if self != t
        save_state(self)
        self = t
        load_state(self)
      stats()
      ERET
      
      
    
    
    
  
  